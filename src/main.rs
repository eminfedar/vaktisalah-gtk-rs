use std::time::Duration;

use adw::{Application, Toast};
use gio::prelude::ApplicationExt;
use gio::Notification;
use gtk::prelude::ObjectExt;
use gtk::prelude::{ButtonExt, GtkApplicationExt, GtkWindowExt, WidgetExt};
use gtk::subclass::prelude::*;
use gtk::PropertyExpression;

use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::tokio;
use relm4::AsyncComponentSender;
use relm4::{adw, gtk, gtk::gio, gtk::glib, RelmApp};

// Crate
mod listitem;
mod networking;
mod prayer;
mod preferences;
mod sound;
mod ui_relm;
//mod translations;

use crate::listitem::ListItemIDName;
use crate::networking::*;
use crate::prayer::*;
use crate::preferences::*;
use crate::sound::*;
use crate::ui_relm::*;

glib::wrapper! {
    pub struct ListItemIDNameGtk(ObjectSubclass<ListItemIDName>);
}
impl ListItemIDNameGtk {
    pub fn new(item_id: &str, item_name: &str) -> Self {
        glib::Object::builder()
            .property("itemId", item_id.to_string())
            .property("itemName", item_name.to_string())
            .build()
    }

    pub fn get_item_name(&self) -> String {
        self.property::<String>("itemName")
    }
    pub fn get_item_id(&self) -> String {
        self.property::<String>("itemId")
    }
}

#[derive(Debug)]
#[tracker::track]
struct App {
    preferences_json: PreferencesJson,

    // Lists
    country_list_model: gio::ListStore,
    city_list_model: gio::ListStore,
    district_list_model: gio::ListStore,

    // MainWindow
    toast_message: &'static str,
    current_page: &'static str,

    // Prayer times
    todays_prayers: Option<PrayerTimesWithDate>,
    tomorrows_prayers: Option<PrayerTimesWithDate>,
    remaining_time: Option<RemainingTime>,
    next_prayer: Option<Prayer>,
}

#[derive(Debug)]
enum CommandMessage {
    SecondTick,
}

#[derive(Debug)]
enum Message {
    PageMain,
    PageSettings,

    DarkMode,

    SaveSettings,

    CountryChanged(Option<glib::Object>),
    CityChanged(Option<glib::Object>),
    DistrictChanged(Option<glib::Object>),

    WarningMinutesChanged(u8),
}

fn liststore_from_vec(items: &[ListItemIDNameGtk]) -> gio::ListStore {
    let model = gio::ListStore::with_type(glib::Type::OBJECT);
    model.extend_from_slice(items);

    model
}

fn get_model_and_selected_position(
    value: &serde_json::Value,
    selected_string: &str,
) -> (gio::ListStore, u32) {
    let list = PreferencesJson::value_to_listitem(value);
    let list_model = liststore_from_vec(&list);
    let selected_position = list
        .iter()
        .position(|item| item.get_item_name() == selected_string)
        .unwrap_or(0) as u32;

    (list_model, selected_position)
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = PreferencesJson;
    type Input = Message;
    type Output = ();
    type CommandOutput = CommandMessage;

    view! {
        adw::Window {
            set_title: Some("Vakt-i Salah"),
            set_default_width: 225,
            set_default_height: 360,
            set_icon_name: Some("vaktisalah"),

            set_resizable: false,
            set_hide_on_close: true,

            #[template]
            MainWindow {
                // == Settings Page Events
                #[template_child]
                settings_page.btn_go_main { connect_clicked => Message::PageMain },

                #[template_child]
                settings_page.btn_dark_mode { connect_clicked => Message::DarkMode },

                #[template_child]
                settings_page.btn_save { connect_clicked => Message::SaveSettings },

                #[template_child]
                settings_page.dd_country {
                    set_expression: Some(PropertyExpression::new(
                        ListItemIDName::type_(),
                        None::<&gtk::Expression>,
                        "itemName",
                    )),

                    #[track = "model.changed(App::country_list_model())"]
                    set_model: Some(&model.country_list_model),
                    set_selected: selected_country_position,

                    connect_selected_item_notify[sender] => move |dd| { sender.input(Message::CountryChanged(dd.selected_item())); }
                },

                #[template_child]
                settings_page.dd_city {
                    set_expression: Some(PropertyExpression::new(
                        ListItemIDName::type_(),
                        None::<&gtk::Expression>,
                        "itemName",
                    )),

                    #[track = "model.changed(App::city_list_model())"]
                    set_model: Some(&model.city_list_model),
                    set_selected: selected_city_position,

                    connect_selected_item_notify[sender] => move |dd| { sender.input(Message::CityChanged(dd.selected_item())); }
                },

                #[template_child]
                settings_page.dd_district {
                    set_expression: Some(PropertyExpression::new(
                        ListItemIDName::type_(),
                        None::<&gtk::Expression>,
                        "itemName",
                    )),


                    #[track = "model.changed(App::district_list_model())"]
                    set_model: Some(&model.district_list_model),
                    set_selected: selected_district_position,

                    connect_selected_item_notify[sender] => move |dd| { sender.input(Message::DistrictChanged(dd.selected_item())); }
                },

                #[template_child]
                settings_page.spn_warning_minutes {
                    connect_value_changed[sender] => move |btn| { sender.input(Message::WarningMinutesChanged(btn.value_as_int() as u8))}
                },


                // == Main Page Events
                #[template_child]
                main_page.btn_go_settings { connect_clicked => Message::PageSettings },

                #[template_child]
                main_page.lbl_district_name {
                    #[watch]
                    set_label: &model.preferences_json.preferences.district
                },

                #[template_child]
                main_page.lbl_city_name {
                    #[track = "model.changed(App::preferences_json())"]
                    set_label: &model.preferences_json.preferences.city
                },

                // Prayer Times
                #[template_child]
                main_page.prayer_times.lbl_fajr_time {
                    #[track = "model.changed(App::todays_prayers())"]
                    set_label: model.todays_prayers.as_ref().map_or("--:--", |v| v.Imsak.as_str())
                },
                #[template_child]
                main_page.prayer_times.lbl_sunrise_time {
                    #[track = "model.changed(App::todays_prayers())"]
                    set_label: model.todays_prayers.as_ref().map_or("--:--", |v| v.Gunes.as_str())
                },
                #[template_child]
                main_page.prayer_times.lbl_dhuhr_time {
                    #[track = "model.changed(App::todays_prayers())"]
                    set_label: model.todays_prayers.as_ref().map_or("--:--", |v| v.Ogle.as_str())
                },
                #[template_child]
                main_page.prayer_times.lbl_asr_time {
                    #[track = "model.changed(App::todays_prayers())"]
                    set_label: model.todays_prayers.as_ref().map_or("--:--", |v| v.Ikindi.as_str())
                },
                #[template_child]
                main_page.prayer_times.lbl_maghrib_time {
                    #[track = "model.changed(App::todays_prayers())"]
                    set_label: model.todays_prayers.as_ref().map_or("--:--", |v| v.Aksam.as_str())
                },
                #[template_child]
                main_page.prayer_times.lbl_isha_time {
                    #[track = "model.changed(App::todays_prayers())"]
                    set_label: model.todays_prayers.as_ref().map_or("--:--", |v| v.Yatsi.as_str())
                },

                // Date
                #[template_child]
                main_page.lbl_date_hijri {
                    #[track = "model.changed(App::todays_prayers())"]
                    set_label: model.todays_prayers.as_ref().map_or("", |v| v.HicriTarihUzun.as_str()),
                    set_tooltip_text: model.todays_prayers.as_ref().map(|v| v.HicriTarihKisa.as_str() )
                },

                #[template_child]
                main_page.remaining_time_footer.lbl_prayer_time_name {
                    #[track = "model.changed(App::next_prayer())"]
                    set_label: model.next_prayer.as_ref().map_or(String::from("---"), |v| v.to_string()).as_str()
                },

                #[template_child]
                main_page.remaining_time_footer.lbl_prayer_time {
                    #[track = "model.changed(App::remaining_time())"]
                    set_label: model.remaining_time.as_ref().map_or(String::from("--:--:--"), |r| format!(
                        "{:0>2}:{:0>2}:{:0>2}",
                        r.hours, r.minutes, r.seconds
                    )).as_str()
                },


                // == Main Window Events
                #[template_child]
                stk_pages {
                    #[track = "model.changed(App::current_page())"]
                    set_visible_child_name: model.current_page,
                },

                #[template_child]
                toast_overlay {
                    #[track = "model.changed(App::toast_message())"]
                    add_toast: Toast::builder().title(model.toast_message).timeout(3).build()
                }
            },
        }
    }

    async fn init(
        preferences_json: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        // Dark Mode
        if let Some(b) = preferences_json.preferences.dark_mode {
            let style = adw::StyleManager::default();
            if b {
                style.set_color_scheme(adw::ColorScheme::ForceDark)
            } else {
                style.set_color_scheme(adw::ColorScheme::ForceLight)
            }
        }

        // Get Country List
        let (country_list_model, selected_country_position) = get_model_and_selected_position(
            &preferences_json.countries,
            &preferences_json.preferences.country,
        );

        // Get City List
        let (city_list_model, selected_city_position) = get_model_and_selected_position(
            &preferences_json.cities,
            &preferences_json.preferences.city,
        );

        // Get District List
        let (district_list_model, selected_district_position) = get_model_and_selected_position(
            &preferences_json.districts,
            &preferences_json.preferences.district,
        );

        // Calculate remaining time
        let todays_prayers = prayer::get_prayer_times_with_date(&preferences_json, 0);
        dbg!(&todays_prayers);
        let tomorrows_prayers = prayer::get_prayer_times_with_date(&preferences_json, 1);
        let remaining_time = prayer::calculate_remaining_time(&todays_prayers, &tomorrows_prayers);
        let next_prayer = remaining_time.map(|v| v.next_prayer);

        // Init the App
        let model = Self {
            current_page: "main",
            preferences_json,

            country_list_model,
            city_list_model,
            district_list_model,

            toast_message: "",

            next_prayer,
            remaining_time,
            todays_prayers,
            tomorrows_prayers,

            tracker: 0,
        };

        let widgets = view_output!();

        // Check if prayer times on .json file still up to date
        if !prayer::is_prayer_times_valid(&model.preferences_json) {
            println!("Prayer times are not valid!");
            sender.input(Message::SaveSettings);
        }

        // Start second tick
        sender.oneshot_command(async { CommandMessage::SecondTick });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match msg {
            Message::DarkMode => {
                let style = adw::StyleManager::default();
                if style.color_scheme() == adw::ColorScheme::ForceDark {
                    style.set_color_scheme(adw::ColorScheme::ForceLight);
                    self.preferences_json.preferences.dark_mode = Some(false);
                } else {
                    style.set_color_scheme(adw::ColorScheme::ForceDark);
                    self.preferences_json.preferences.dark_mode = Some(true);
                }

                // Save latest preferences struct to the .json file
                match save_preferences_json(&self.preferences_json).await {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("[Error] Failed to save preferences.json: {e:?}");

                        self.set_toast_message("Saving the settings failed.");

                        return;
                    }
                }
            }
            Message::PageMain => self.set_current_page("main"),
            Message::PageSettings => self.set_current_page("settings"),

            Message::CountryChanged(selected_item) => {
                let item = match selected_item {
                    Some(obj) => obj,
                    None => return,
                };

                let item_id: String = item.property("itemId");
                let item_name: String = item.property("itemName");

                println!("Selected country: {item_name} => {item_id}");

                match networking::get_city_list(&item_id).await {
                    Ok(list) => {
                        self.update_preferences_json(|p| {
                            p.cities = list;
                            p.preferences.country = item_name;
                        });

                        let city_list =
                            PreferencesJson::value_to_listitem(&self.preferences_json.cities);

                        self.set_city_list_model(liststore_from_vec(&city_list));
                    }
                    Err(e) => {
                        eprintln!("[Error] while getting city list from network: {e:?}");
                        self.set_toast_message("Network Error.");
                    }
                };
            }
            Message::CityChanged(selected_item) => {
                let item = match selected_item {
                    Some(obj) => obj,
                    None => return,
                };

                let item_id: String = item.property("itemId");
                let item_name: String = item.property("itemName");

                println!("Selected city: {item_name} => {item_id}");

                match networking::get_district_list(&item_id).await {
                    Ok(list) => {
                        self.update_preferences_json(|p| {
                            p.districts = list;
                            p.preferences.city = item_name;
                        });

                        let district_list =
                            PreferencesJson::value_to_listitem(&self.preferences_json.districts);

                        self.set_district_list_model(liststore_from_vec(&district_list));
                    }
                    Err(e) => {
                        eprintln!("[Error] While getting district list from network: {e:?}");
                        self.set_toast_message("Network Error.");
                    }
                };
            }

            Message::DistrictChanged(selected_item) => {
                let item = match selected_item {
                    Some(obj) => obj,
                    None => return,
                };

                let item_id: String = item.property("itemId");
                let item_name: String = item.property("itemName");

                println!("Selected District: {item_name} => {item_id}");

                self.update_preferences_json(|p| {
                    p.preferences.district = item_name;
                    p.preferences.district_id = item_id;
                });
            }

            Message::WarningMinutesChanged(value) => {
                println!("Warning minutes: {value}");

                self.update_preferences_json(|p| {
                    p.preferences.warning_minutes = value;
                });
            }

            Message::SaveSettings => {
                println!("Saving settings!");
                // Update prayer times
                match update_prayer_times_on_network(&mut self.preferences_json).await {
                    Ok(_) => self.set_toast_message("Prayer times updated."),
                    Err(e) => {
                        eprintln!("[Error] Failed to upgrade Prayer Times from internet: {e:?}");

                        self.set_toast_message("Network Error.");

                        return;
                    }
                }

                // Save latest preferences struct to the .json file
                match save_preferences_json(&self.preferences_json).await {
                    Ok(_) => self.set_toast_message("Settings saved."),
                    Err(e) => {
                        eprintln!("[Error] Failed to save preferences.json: {e:?}");

                        self.set_toast_message("Saving the settings failed.");

                        return;
                    }
                }

                self.set_todays_prayers(prayer::get_prayer_times_with_date(
                    &self.preferences_json,
                    0,
                ));
                self.set_tomorrows_prayers(prayer::get_prayer_times_with_date(
                    &self.preferences_json,
                    1,
                ));
                self.set_remaining_time(prayer::calculate_remaining_time(
                    &self.todays_prayers,
                    &self.tomorrows_prayers,
                ));
                self.set_next_prayer(self.remaining_time.map(|v| v.next_prayer));

                self.set_current_page("main");
                self.update_preferences_json(|_| {}); // update signal
            }
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();

        match message {
            CommandMessage::SecondTick => {
                self.set_remaining_time(prayer::calculate_remaining_time(
                    &self.todays_prayers,
                    &self.tomorrows_prayers,
                ));

                if let Some(r) = self.remaining_time.as_ref() {
                    let warn_min = self.preferences_json.preferences.warning_minutes as u32;
                    let current_min = r.hours as u32 * 60 + r.minutes as u32;

                    let should_warn = current_min == warn_min && r.seconds == 0;

                    if should_warn {
                        // Send notification
                        let notif = Notification::new(
                            format!(
                                "{} minutes left {}!",
                                self.preferences_json.preferences.warning_minutes, r.next_prayer
                            )
                            .as_str(),
                        );

                        let app = root.application().unwrap();

                        app.send_notification(Some("prayer-time-warsssning"), &notif);

                        std::thread::spawn(|| {
                            play_alert();
                        });

                        root.present();
                    }
                }

                sender.oneshot_command(async {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    CommandMessage::SecondTick
                });
            }
        }
    }
}

fn on_activate(application: &Application) {
    if application.windows().len() == 1 {
        application.windows().first().as_ref().unwrap().present();
    }
}

fn main() {
    let preferences_json = match read_preferences_json_file() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Failed to read preferences.json: {err:?}");
            return;
        }
    };

    let application = Application::builder()
        .application_id("io.github.eminfedar.vaktisalah-gtk-rs")
        .build();

    application.connect_activate(on_activate);

    let app = RelmApp::from_app(application);
    app.run_async::<App>(preferences_json);
}
