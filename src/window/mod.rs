mod imp;

use std::collections::HashMap;

use adw::subclass::prelude::ObjectSubclassIsExt;
use adw::ButtonRow;
use adw::ComboRow;
use adw::SpinRow;
use async_channel::Receiver;
use chrono::Local;
use chrono::Locale;
use gtk::gio;
use gtk::gio::prelude::ApplicationExt;
use gtk::gio::Notification;
use gtk::glib;
use gtk::glib::object::ObjectExt;
use gtk::glib::ParamSpec;
use gtk::prelude::GtkWindowExt;

use gtk::Button;
use gtk::StringList;

use formatx::formatx;
use gettextrs::gettext;
use gtk::StringObject;
use imp::Message;

use crate::networking;
use crate::prayer;

use crate::sound::play_alert;
use crate::LOCALE;
use crate::RUNTIME;

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

fn fill_model(
    selected_name: Option<String>,
    name_id_list: &HashMap<String, String>,
) -> (StringList, i32) {
    let mut keys: Vec<&str> = name_id_list.keys().map(|key| key.as_str()).collect();

    keys.sort();
    let selected_index = match selected_name {
        Some(name) => keys.binary_search(&name.as_str()).unwrap_or(0) as i32,
        None => 0,
    };
    let string_list = StringList::new(&keys);

    (string_list, selected_index)
}

#[gtk::template_callbacks]
impl MainWindow {
    pub fn new(app: &adw::Application) -> Self {
        let new_self: Self = glib::Object::builder().property("application", app).build();

        let imp = new_self.imp();

        let (tx, rx) = async_channel::bounded(4);
        imp.sender.replace(Some(tx));

        new_self.listen_channel_message(rx);

        new_self
    }

    pub fn read_preferences(&self) {
        let imp = self.imp();
        let pref = imp.preferences.borrow();

        self.update_model_country(
            pref.countries.borrow().clone(),
            pref.countries_en.borrow().clone(),
            Some(pref.preferences.country.borrow().clone()),
        );
        self.update_model_city(
            pref.cities.borrow().clone(),
            Some(pref.preferences.city.borrow().clone()),
        );
        let district = pref.preferences.district.borrow();
        self.update_model_district(pref.districts.borrow().clone(), Some(district.clone()));
        self.set_district_title(district.clone());

        // Warn Min
        let warn_min = *pref.preferences.warning_minutes.borrow();
        self.set_warn_min(f64::from(warn_min));

        // Set Prayer Time Labels:
        self.update_prayer_time_labels();
    }

    pub fn update_prayer_time_labels(&self) {
        let imp = self.imp();
        let pref = imp.preferences.borrow();

        // Read Today's Prayers:
        let todays_prayers = prayer::get_prayers_of_day(&pref, 0);
        let tomorrows_prayers = prayer::get_prayers_of_day(&pref, 1);

        // Set labels
        if let Some(today) = todays_prayers.clone() {
            self.set_time_fajr(today.Imsak);
            self.set_time_sunrise(today.Gunes);
            self.set_time_dhuhr(today.Ogle);
            self.set_time_asr(today.Ikindi);
            self.set_time_maghrib(today.Aksam);
            self.set_time_isha(today.Yatsi);

            let gregorian_date = Local::now()
                .format_localized("%d %B %Y", *LOCALE)
                .to_string();
            let hijri_date = today.HicriTarihUzun;
            self.set_gregorian_date(gregorian_date);
            self.set_hijri_date(hijri_date);
        }

        imp.todays_prayers.replace(todays_prayers);
        imp.tomorrows_prayers.replace(tomorrows_prayers);
    }

    fn update_model_country(
        &self,
        countries: HashMap<String, String>,
        countries_en: HashMap<String, String>,
        selected_country: Option<String>,
    ) {
        let imp = self.imp();
        imp.countries.replace(countries.clone());
        imp.countries_en.replace(countries_en.clone());
        if let Some(c) = selected_country.clone() {
            imp.country.replace(c);
        }

        let (list, selected_index) = if *LOCALE == Locale::tr_TR {
            fill_model(selected_country.clone(), &countries)
        } else {
            fill_model(selected_country.clone(), &countries_en)
        };

        self.set_model_country(list);
        self.set_selected_country_index(selected_index);
    }

    fn update_model_city(&self, cities: HashMap<String, String>, selected_city: Option<String>) {
        let imp = self.imp();
        imp.cities.replace(cities.clone());
        if let Some(c) = selected_city.clone() {
            imp.city.replace(c);
        }

        // City
        let (list, selected_index) = fill_model(selected_city.clone(), &cities);

        self.set_model_city(list);
        self.set_selected_city_index(selected_index);
    }

    fn update_model_district(
        &self,
        districts: HashMap<String, String>,
        selected_district: Option<String>,
    ) {
        let imp = self.imp();
        imp.districts.replace(districts.clone());
        if let Some(c) = selected_district.clone() {
            imp.district.replace(c);
        }

        // District
        let (list, selected_index) = fill_model(selected_district.clone(), &districts);

        self.set_model_district(list);
        self.set_selected_district_index(selected_index);
    }

    pub fn init_second_tick(&self) {
        let imp = self.imp();
        let pref = imp.preferences.borrow();

        // Check if prayer times on .json file still up to date
        if !prayer::is_prayer_times_valid(&pref) {
            println!("Prayer times are not valid, updating...");

            let district_id = pref.preferences.district_id.borrow().clone();
            let sender = imp.sender.borrow().clone().unwrap();

            RUNTIME.spawn(async move {
                let result = networking::get_prayer_times(&district_id).await;
                sender
                    .send(Message::PrayerTimesArrived(result))
                    .await
                    .unwrap();
            });
        }

        self.on_second_tick();

        let self_clone = self.downgrade();

        println!("Starting tick...");

        glib::timeout_add_seconds_local(1, move || {
            let self_clone = self_clone.upgrade().unwrap();
            self_clone.on_second_tick();

            glib::ControlFlow::Continue
        });
    }

    pub fn on_second_tick(&self) {
        let imp = self.imp();
        let pref = &imp.preferences.borrow().preferences;

        // Update current time
        let now = Local::now();
        let current_time_str = now.format("%H:%M").to_string();
        self.set_current_time(current_time_str);

        let remaining_time = prayer::calculate_remaining_time(
            &imp.todays_prayers.borrow(),
            &imp.tomorrows_prayers.borrow(),
        );

        if let Some(r) = remaining_time.as_ref() {
            let current_min = r.hours as u32 * 60 + r.minutes as u32;

            // Update labels:
            let time_format = format!("{:0>2}:{:0>2}:{:0>2}", r.hours, r.minutes, r.seconds);
            self.set_next_prayer_name(r.next_prayer.to_string());
            self.set_next_prayer_time(time_format);
            let current_prayer: u8 = r.next_prayer as u8;
            self.set_current_prayer(current_prayer as i32);

            // Warning Time check:
            let warn_min = *pref.warning_minutes.borrow() as u32;
            let should_warn = current_min == warn_min && r.seconds == 0;

            if should_warn {
                // Send notification
                let msg = formatx!(
                    gettext("{min} minutes left {prayer}"),
                    min = warn_min,
                    prayer = r.next_prayer
                )
                .unwrap();

                let notif = Notification::new(&msg);

                self.application()
                    .unwrap()
                    .send_notification(Some("prayer-time-warn"), &notif);

                std::thread::spawn(|| {
                    play_alert();
                });

                self.present();
            }
        }
    }

    pub fn listen_channel_message(&self, receiver: Receiver<Message>) {
        let imp = self.imp().downgrade();
        let self_clone = self.downgrade();

        glib::spawn_future_local(async move {
            let imp = imp.upgrade().unwrap();
            let self_clone = self_clone.upgrade().unwrap();

            loop {
                match receiver.recv().await {
                    Ok(m) => match m {
                        Message::CityListArrived(result, _country) => match result {
                            Ok(r) => {
                                println!("City List Arrived: {r:?}");
                                self_clone.update_model_city(r, None);
                            }
                            Err(e) => eprintln!("Failed to fetch cities: {e}"),
                        },
                        Message::DistrictListArrived(result, _city) => match result {
                            Ok(r) => {
                                println!("District List Arrived: {r:?}");
                                self_clone.update_model_district(r, None);
                            }
                            Err(e) => eprintln!("Failed to fetch districts: {e}"),
                        },
                        Message::PrayerTimesArrived(result) => match result {
                            Ok(v) => {
                                let pref = imp.preferences.borrow().clone();
                                println!("Prayer Times Arrived");
                                let mut hm = HashMap::new();
                                for day in v {
                                    let key = day.MiladiTarihKisa.clone();

                                    hm.insert(key, day);
                                }

                                // Save latest preferences struct to the .json file
                                pref.cities.replace(imp.cities.borrow().clone());
                                pref.districts.replace(imp.districts.borrow().clone());

                                pref.preferences
                                    .country
                                    .replace(imp.country.borrow().clone());
                                pref.preferences.city.replace(imp.city.borrow().clone());

                                let district = imp.district.borrow().clone();
                                let district_id =
                                    imp.districts.borrow().get(&district).unwrap().clone();

                                self_clone.set_district_title(district.clone());
                                pref.preferences.district.replace(district);
                                pref.preferences.district_id.replace(district_id);

                                pref.prayer_times.replace(hm);
                                pref.save().unwrap();

                                // Update models
                                let todays_prayers = prayer::get_prayers_of_day(&pref, 0);
                                let tomorrows_prayers = prayer::get_prayers_of_day(&pref, 1);

                                // Set labels
                                if let Some(today) = todays_prayers.clone() {
                                    self_clone.set_time_fajr(today.Imsak);
                                    self_clone.set_time_sunrise(today.Gunes);
                                    self_clone.set_time_dhuhr(today.Ogle);
                                    self_clone.set_time_asr(today.Ikindi);
                                    self_clone.set_time_maghrib(today.Aksam);
                                    self_clone.set_time_isha(today.Yatsi);

                                    let gregorian_date = Local::now()
                                        .format_localized("%d %B %Y", *LOCALE)
                                        .to_string();

                                    let hijri_date = today.HicriTarihUzun;
                                    self_clone.set_gregorian_date(gregorian_date);
                                    self_clone.set_hijri_date(hijri_date);
                                }

                                imp.todays_prayers.replace(todays_prayers);
                                imp.tomorrows_prayers.replace(tomorrows_prayers);
                                imp.preferences.replace(pref);
                                imp.visible_day.replace(0);

                                self_clone.on_second_tick();

                                let toast = adw::Toast::new(&gettext("Prayer Times Updated."));
                                imp.toast_overlay.add_toast(toast);

                                imp.navigation_view.pop_to_page(&imp.main_page.get());

                                println!("Prayer Times updated!");
                            }
                            Err(e) => {
                                let toast =
                                    adw::Toast::new(&gettext("Failed to get prayer times!"));
                                imp.toast_overlay.add_toast(toast);

                                eprintln!("Failed to fetch prayer times: {e}");
                            }
                        },
                    },
                    Err(e) => eprintln!("listen_channel_message ERROR: {e}"),
                }
            }
        });
    }

    #[template_callback]
    fn if_style(&self, prayer_number: i32, current_prayer: i32) -> Vec<String> {
        if prayer_number == (current_prayer % 6) {
            vec!["success".to_string(), "title-3".to_string()]
        } else {
            Vec::new()
        }
    }

    #[template_callback]
    fn on_country_changed(&self, param: ParamSpec, row: ComboRow) {
        let index = self.selected_country_index();

        if index == -1 {
            return;
        }
        let value_obj: StringObject = row.property(param.name());
        let value: String = value_obj.string().to_string();

        let imp = self.imp();
        if value == *imp.country.borrow() {
            return;
        }

        let country_id = match imp.countries.borrow().get(&value) {
            Some(v) => v.clone(),
            None => {
                let b = imp.countries_en.borrow();
                b.get(&value).unwrap().clone()
            }
        };
        let country_name_clone = value.clone();

        imp.country.replace(value);

        let sender = imp.sender.borrow().clone().unwrap();

        let toast = adw::Toast::builder()
            .title(gettext("Getting Cities..."))
            .timeout(1)
            .build();
        imp.toast_overlay.add_toast(toast);

        RUNTIME.spawn(async move {
            let result = networking::get_city_list(&country_id).await;
            sender
                .send(Message::CityListArrived(result, country_name_clone))
                .await
                .unwrap();
        });
    }

    #[template_callback]
    fn on_city_changed(&self, param: ParamSpec, row: ComboRow) {
        let index = self.selected_city_index();

        if index == -1 {
            return;
        }

        let value_obj: StringObject = row.property(param.name());
        let value: String = value_obj.string().to_string();

        let imp = self.imp();
        if value == *imp.city.borrow() {
            return;
        }

        let city_id = match imp.cities.borrow().get(&value) {
            Some(v) => v.clone(),
            None => String::new(),
        };
        let city_name_clone = value.clone();
        imp.city.replace(value);

        let sender = imp.sender.borrow().clone().unwrap().clone();

        let toast = adw::Toast::builder()
            .title(gettext("Getting Districts..."))
            .timeout(1)
            .build();
        imp.toast_overlay.add_toast(toast);

        RUNTIME.spawn(async move {
            let result = networking::get_district_list(&city_id).await;
            sender
                .send(Message::DistrictListArrived(result, city_name_clone))
                .await
                .unwrap();
        });
    }

    #[template_callback]
    fn on_district_changed(&self, param: ParamSpec, row: ComboRow) {
        let index = self.selected_district_index();

        if index == -1 {
            return;
        }

        let value_obj: StringObject = row.property(param.name());
        let value: String = value_obj.string().to_string();

        let imp = self.imp();
        if value == *imp.district.borrow() {
            return;
        }

        imp.district.replace(value);
    }

    #[template_callback]
    fn on_warn_min_changed(&self, param: ParamSpec, spin: SpinRow) {
        let value: f64 = spin.property(param.name());

        let imp = self.imp();
        let pref = imp.preferences.borrow();

        pref.preferences.warning_minutes.replace(value as u8);
        pref.save().unwrap();
    }

    #[template_callback]
    fn on_update_prayer_times_activated(&self, _button: ButtonRow) {
        let imp = self.imp();

        let district = imp.district.borrow().clone();
        let district_id = imp.districts.borrow().get(&district).unwrap().clone();

        let sender = imp.sender.borrow().clone().unwrap();

        let toast = adw::Toast::builder()
            .title(gettext("Getting Prayer Times..."))
            .timeout(1)
            .build();
        imp.toast_overlay.add_toast(toast);

        RUNTIME.spawn(async move {
            let result = networking::get_prayer_times(&district_id).await;
            sender
                .send(Message::PrayerTimesArrived(result))
                .await
                .unwrap();
        });
    }

    // TODO: Add this feature later
    #[template_callback]
    fn on_btn_prev_date_clicked(&self, _button: Button) {
        let imp = self.imp();
        imp.visible_day.replace_with(|&mut old| old - 1);
    }

    #[template_callback]
    fn on_btn_next_date_clicked(&self, _button: Button) {
        let imp = self.imp();
        imp.visible_day.replace_with(|&mut old| old + 1);
    }
}
