use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use crate::networking::{self, update_prayer_times_on_network, PrayerTimesWithDate};
use crate::prayer::{Prayer, RemainingTime};
use crate::preferences::{save_preferences_json, PreferencesJson};
use crate::sound::play_alert;
use crate::translations::translate_ui;
use adw::{Application, ColorScheme, Toast, ToastOverlay};
use chrono::{Days, Local, Utc};
use gio::subclass::prelude::ObjectSubclassType;
use gio::Notification;
use glib::{timeout_add_local_once, Object};

use gtk::{prelude::*, PropertyExpression, SpinButton};
use gtk::{ApplicationWindow, Builder, DropDown, Stack};
use gtk::{Button, Label};
use gtk4 as gtk;
use simple_localization::tr;

use crate::listitem::ListItemIDName;

static NEXT_PRAYER: AtomicU8 = AtomicU8::new(0);

// ==========  UI BUILDING  ===========
pub fn build_ui(application: &Application, preferences_json: PreferencesJson) {
    let ui_src = include_str!("../ui/MainWindow.ui");
    let builder = Builder::from_string(ui_src);

    let window: ApplicationWindow = builder.object("window").unwrap();
    window.set_application(Some(application));

    // Setup UI
    setup_stack_navigation_ui(&builder);
    setup_dark_mode_toggle(&builder);

    let preferences_json_rc = Rc::new(RefCell::new(preferences_json));
    setup_settings_ui(&builder, Rc::clone(&preferences_json_rc));
    setup_city_labels(&builder, Rc::clone(&preferences_json_rc));
    translate_ui(&builder);

    // Show window
    window.show();

    // Update UI every second
    let remaining_time = calculate_remaining_time(Rc::clone(&preferences_json_rc));
    update_ui(Rc::clone(&preferences_json_rc), &builder, &remaining_time);

    let app = application.clone();

    glib::timeout_add_local(Duration::from_secs(1), move || {
        let remaining_time = calculate_remaining_time(Rc::clone(&preferences_json_rc));

        let window = app.active_window().unwrap();

        if window.is_visible() {
            update_ui(Rc::clone(&preferences_json_rc), &builder, &remaining_time);
        }

        let should_warn_user: bool = should_warn(Rc::clone(&preferences_json_rc), &remaining_time);

        if should_warn_user {
            let total_minutes =
                (remaining_time.hours as u32) * 60 + (remaining_time.minutes as u32);

            play_alert();

            // Send notifications
            let notif = Notification::new(format!("{} minutes left!", total_minutes).as_str());
            app.send_notification(Some("prayer-time-warning"), &notif);
        }

        glib::Continue(true)
    });
}

fn setup_stack_navigation_ui(builder: &Builder) {
    // Page Navigation
    let stk_pages: Stack = builder.object("stk_pages").unwrap();
    let stk_pages_rc = Rc::new(stk_pages);

    let stk_pages_clone = Rc::clone(&stk_pages_rc);
    let btn_settings: Button = builder.object("btn_settings").unwrap();
    btn_settings.connect_clicked(move |_| {
        stk_pages_clone.set_visible_child_name("settings");
    });

    let stk_pages_clone = Rc::clone(&stk_pages_rc);
    let btn_to_mainscreen: Button = builder.object("btn_to_mainscreen").unwrap();
    btn_to_mainscreen.connect_clicked(move |_| {
        stk_pages_clone.set_visible_child_name("main");
    });
}

fn setup_dark_mode_toggle(builder: &Builder) {
    // Dark-Light mode toggle button
    let btn_theme_toggle: Button = builder.object("btn_theme_toggle").unwrap();
    btn_theme_toggle.connect_clicked(move |_| {
        let current = adw::StyleManager::default();
        if current.color_scheme() == ColorScheme::ForceDark {
            current.set_color_scheme(ColorScheme::ForceLight);
        } else {
            current.set_color_scheme(ColorScheme::ForceDark);
        }
    });
}

glib::wrapper! {
    pub struct ListItemIDNameGtk(ObjectSubclass<ListItemIDName>);
}
impl ListItemIDNameGtk {
    pub fn new(item_id: &str, item_name: &str) -> Self {
        Object::builder()
            .property("itemId", item_id.to_string())
            .property("itemName", item_name.to_string())
            .build()
    }
}

fn setup_settings_ui(builder: &Builder, preferences_json_rc_cell: Rc<RefCell<PreferencesJson>>) {
    // Settings:
    let preferences_json = preferences_json_rc_cell.as_ref().borrow();
    let property_expression = Some(PropertyExpression::new(
        ListItemIDName::type_(),
        None::<&gtk::Expression>,
        "itemName",
    ));

    // -- Set Warning Minutes
    let spn_warning_minutes: SpinButton = builder.object("spn_warning_minutes").unwrap();
    spn_warning_minutes.set_value(preferences_json.preferences.warning_minutes.into());

    // -- Fill Countries
    let country_list: Vec<ListItemIDNameGtk> = preferences_json
        .countries
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| ListItemIDNameGtk::new(v.as_str().unwrap(), k))
        .collect();
    let country_list_model = gio::ListStore::new(ListItemIDNameGtk::static_type());
    country_list_model.extend_from_slice(&country_list);
    let selected_country_position = country_list
        .iter()
        .position(|x| x.property::<String>("itemName") == preferences_json.preferences.country)
        .unwrap_or(0);

    let dd_country: DropDown = builder.object("dd_country").unwrap();
    dd_country.set_expression(property_expression.clone());
    dd_country.set_model(Some(&country_list_model));
    dd_country.set_selected(selected_country_position as u32);

    // -- Fill Cities
    let city_list: Vec<ListItemIDNameGtk> = preferences_json
        .cities
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| ListItemIDNameGtk::new(v.as_str().unwrap(), k))
        .collect();
    let city_list_model = gio::ListStore::new(ListItemIDNameGtk::static_type());
    city_list_model.extend_from_slice(&city_list);
    let selected_city_position = city_list
        .iter()
        .position(|x| x.property::<String>("itemName") == preferences_json.preferences.city)
        .unwrap_or(0);

    let dd_city: DropDown = builder.object("dd_city").unwrap();
    dd_city.set_expression(property_expression.clone());
    dd_city.set_model(Some(&city_list_model));
    dd_city.set_selected(selected_city_position as u32);

    // -- Fill Districts
    let district_list: Vec<ListItemIDNameGtk> = preferences_json
        .districts
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| ListItemIDNameGtk::new(v.as_str().unwrap(), k))
        .collect();
    let district_list_model = gio::ListStore::new(ListItemIDNameGtk::static_type());
    district_list_model.extend_from_slice(&district_list);
    let selected_district_position = district_list
        .iter()
        .position(|x| x.property::<String>("itemId") == preferences_json.preferences.district_id)
        .unwrap_or(0);

    let dd_district: DropDown = builder.object("dd_district").unwrap();
    dd_district.set_expression(property_expression);
    dd_district.set_model(Some(&district_list_model));
    dd_district.set_selected(selected_district_position as u32);

    // == CONNECT SIGNALS
    let preferences_json_rc_clone = Rc::clone(&preferences_json_rc_cell);
    let dd_city_rc = Rc::new(dd_city);
    let dd_city_rc_clone = Rc::clone(&dd_city_rc);

    let toast_overlay: ToastOverlay = builder.object("toast_overlay").unwrap();
    let toast_overlay_rc = Rc::new(toast_overlay);
    let toast_overlay_rc_clone = Rc::clone(&toast_overlay_rc);

    dd_country.connect_selected_item_notify(move |dd| {
        let item = dd.selected_item().unwrap();
        let country_id: String = item.property("itemId");

        // Clone RCs to send it in timeout
        let preferences_json_rc_clone = Rc::clone(&preferences_json_rc_clone);
        let dd_city_rc_clone = Rc::clone(&dd_city_rc_clone);
        let toast_overlay = Rc::clone(&toast_overlay_rc_clone);

        // Use timeout_add_local_once to get prayer times without blocking.
        timeout_add_local_once(Duration::from_secs(0), move || {
            match networking::get_city_list(&country_id) {
                Ok(value) => {
                    let mut p = (*preferences_json_rc_clone).borrow_mut();
                    p.cities = value;

                    let city_list: Vec<ListItemIDNameGtk> = p
                        .districts
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| ListItemIDNameGtk::new(v.as_str().unwrap(), k))
                        .collect();
                    let city_list_model = gio::ListStore::new(ListItemIDNameGtk::static_type());
                    city_list_model.extend_from_slice(&city_list);

                    dd_city_rc_clone.set_model(Some(&city_list_model));
                }
                Err(e) => {
                    eprintln!("[Error] while getting district list from network:\n{}", e);
                    toast_overlay.add_toast(Toast::new("Network Error."));
                    toast_overlay.show();
                }
            };
        });
    });

    let dd_city_rc_clone = Rc::clone(&dd_city_rc);
    let dd_district_rc = Rc::new(dd_district);
    let dd_district_rc_clone = Rc::clone(&dd_district_rc);
    let preferences_json_rc_clone = Rc::clone(&preferences_json_rc_cell);
    let toast_overlay_rc_clone = Rc::clone(&toast_overlay_rc);

    dd_city_rc_clone.connect_selected_item_notify(move |dd| {
        let item = dd.selected_item().unwrap();
        let city_id: String = item.property("itemId");

        // Clone RCs to send it in timeout
        let preferences_json_rc_clone = Rc::clone(&preferences_json_rc_clone);
        let dd_district_rc_clone = Rc::clone(&dd_district_rc_clone);
        let toast_overlay = Rc::clone(&toast_overlay_rc_clone);

        // Use timeout_add_local_once to get prayer times without blocking.
        timeout_add_local_once(Duration::from_secs(0), move || {
            match networking::get_district_list(&city_id) {
                Ok(value) => {
                    let mut p = (*preferences_json_rc_clone).borrow_mut();
                    p.districts = value;

                    let district_list: Vec<ListItemIDNameGtk> = p
                        .districts
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| ListItemIDNameGtk::new(v.as_str().unwrap(), k))
                        .collect();
                    let district_list_model = gio::ListStore::new(ListItemIDNameGtk::static_type());
                    district_list_model.extend_from_slice(&district_list);

                    dd_district_rc_clone.set_model(Some(&district_list_model));
                }
                Err(e) => {
                    eprintln!("[Error] while getting district list from network:\n{}", e);
                    toast_overlay.add_toast(Toast::new("Network Error."));
                    toast_overlay.show();
                }
            };
        });
    });

    let preferences_json_rc_clone = Rc::clone(&preferences_json_rc_cell);
    let dd_district_rc_clone = Rc::clone(&dd_district_rc);
    let dd_city_rc_clone = Rc::clone(&dd_city_rc);

    let builder_clone = builder.clone();

    let stk_pages: Stack = builder.object("stk_pages").unwrap();
    stk_pages.set_visible_child_name("main");

    let btn_save: Button = builder.object("btn_save").unwrap();
    let toast_overlay: ToastOverlay = builder.object("toast_overlay").unwrap();

    btn_save.connect_clicked(move |_btn| {
        let mut p = (*preferences_json_rc_clone).borrow_mut();

        let district_selected_item = dd_district_rc_clone.selected_item().unwrap();
        let selected_district_id: String = district_selected_item.property("itemId");

        if p.preferences.district_id == selected_district_id
            && p.preferences.warning_minutes == spn_warning_minutes.value_as_int() as u8
        {
            return;
        }

        // District changed. Update everything
        let city_selected_item = dd_city_rc_clone.selected_item().unwrap();
        p.preferences.city = city_selected_item.property("itemName");

        p.preferences.district_id = district_selected_item.property("itemId");
        p.preferences.district = district_selected_item.property("itemName");

        p.preferences.warning_minutes = spn_warning_minutes.value_as_int() as u8;

        // Update & Save new times
        match update_prayer_times_on_network(&mut p) {
            Ok(_) => (),
            Err(e) => {
                eprintln!(
                    "[Error] Failed to upgrade Prayer Times from internet:\n{}",
                    e
                );

                toast_overlay.add_toast(Toast::new("Network Error."));
                toast_overlay.show();

                return;
            }
        }
        match save_preferences_json(&p) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("[Error] Failed to save preferences.json:\n{}", e);

                toast_overlay.add_toast(Toast::new("Saving the settings failed."));
                toast_overlay.show();

                return;
            }
        };

        // Return to main screen
        drop(p); // dropped mutable reference

        let p_clone = Rc::clone(&preferences_json_rc_clone);
        setup_city_labels(&builder_clone, p_clone);
        stk_pages.set_visible_child_name("main");
    });
}

fn setup_city_labels(builder: &Builder, preferences_json_rc_cell: Rc<RefCell<PreferencesJson>>) {
    let preferences_json = preferences_json_rc_cell.as_ref().borrow();

    // Fill city labels
    let lbl_city: Label = builder.object("lbl_city").unwrap();
    let lbl_district: Label = builder.object("lbl_district").unwrap();

    lbl_city.set_text(&preferences_json.preferences.city);
    lbl_district.set_text(&preferences_json.preferences.district);
}

fn should_warn(
    preferences_json_rc_cell: Rc<RefCell<PreferencesJson>>,
    remaining_time: &RemainingTime,
) -> bool {
    let preferences_json = preferences_json_rc_cell.as_ref().borrow();

    let warn_min = preferences_json.preferences.warning_minutes as u32;
    let current_min = remaining_time.hours as u32 * 60 + remaining_time.minutes as u32;

    current_min == warn_min && remaining_time.seconds == 0
}

fn calculate_remaining_time(
    preferences_json_rc_cell: Rc<RefCell<PreferencesJson>>,
) -> RemainingTime {
    let preferences_json = preferences_json_rc_cell.as_ref().borrow();

    // Get prayer times from json file
    let today_formatted = Utc::now().format("%d.%m.%Y").to_string();
    let today_prayers: PrayerTimesWithDate = serde_json::from_value(
        preferences_json
            .prayer_times
            .get(&today_formatted)
            .unwrap()
            .to_owned(),
    )
    .unwrap();

    let tomorrow_formatted = Utc::now()
        .checked_add_days(Days::new(1))
        .unwrap()
        .format("%d.%m.%Y")
        .to_string();
    let tomorrow_prayers: PrayerTimesWithDate = serde_json::from_value(
        preferences_json
            .prayer_times
            .get(&tomorrow_formatted)
            .unwrap()
            .to_owned(),
    )
    .unwrap();

    // Calculate Remaning Time
    let today_prayer_times_array = vec![
        &today_prayers.Imsak,
        &today_prayers.Gunes,
        &today_prayers.Ogle,
        &today_prayers.Ikindi,
        &today_prayers.Aksam,
        &today_prayers.Yatsi,
        &tomorrow_prayers.Imsak,
    ];
    let now = Local::now();

    for (i, prayer_time) in today_prayer_times_array.iter().enumerate() {
        let mut hours = (prayer_time[0..2]).parse::<i32>().unwrap();
        let minutes = (prayer_time[3..5]).parse::<i32>().unwrap();

        if i == 6 {
            hours += 24;
        }

        let now_formatted = now.time().format("%H:%M:%S").to_string();
        let now_hours = (now_formatted[0..2]).parse::<i32>().unwrap();
        let now_minutes = (now_formatted[3..5]).parse::<i32>().unwrap();
        let now_seconds = (now_formatted[6..8]).parse::<i32>().unwrap();

        if now_hours > hours {
            continue;
        }

        if now_hours == hours && now_minutes >= minutes {
            continue;
        }

        // Found the next prayer. Calculate remaining time:
        let total_now_seconds = now_hours * 3600 + now_minutes * 60 + now_seconds;
        let total_prayer_seconds = hours * 3600 + minutes * 60;
        let total_remaining_seconds = total_prayer_seconds - total_now_seconds;

        let remaining_seconds = total_remaining_seconds % 60;
        let remaining_minutes = (total_remaining_seconds / 60) % 60;
        let remaining_hours = total_remaining_seconds / 3600;

        return RemainingTime {
            hours: remaining_hours as u8,
            minutes: remaining_minutes as u8,
            seconds: remaining_seconds as u8,
            next_prayer: Prayer::from(i as u8),
        };
    }

    RemainingTime {
        hours: 0,
        minutes: 0,
        seconds: 0,
        next_prayer: Prayer::Fajr,
    }
}

fn update_ui(
    preferences_json_rc_cell: Rc<RefCell<PreferencesJson>>,
    builder: &Builder,
    remaining_time: &RemainingTime,
) {
    let preferences_json = preferences_json_rc_cell.as_ref().borrow();

    // Get prayer times from json file
    let today_formatted = Utc::now().format("%d.%m.%Y").to_string();
    let today_prayers: PrayerTimesWithDate = serde_json::from_value(
        preferences_json
            .prayer_times
            .get(&today_formatted)
            .unwrap()
            .to_owned(),
    )
    .unwrap();

    // Update some stuff after prayer time changed
    if NEXT_PRAYER.load(Ordering::Relaxed) != remaining_time.next_prayer as u8 {
        // Display dates
        let lbl_date_gregorian: Label = builder.object("lbl_date_gregorian").unwrap();
        let lbl_date_hijri: Label = builder.object("lbl_date_hijri").unwrap();

        lbl_date_gregorian.set_text(&Utc::now().format("%d %B %Y").to_string());
        lbl_date_hijri.set_text(&today_prayers.HicriTarihUzun);
        lbl_date_hijri.set_tooltip_text(Some(&today_prayers.HicriTarihKisa));

        // Colorize Current Prayer
        colorize_current_prayer_label(builder, &today_prayers, remaining_time.next_prayer);

        NEXT_PRAYER.store(remaining_time.next_prayer as u8, Ordering::Relaxed);
    }

    // Display Remaining Time
    let remaining_time_str = format!(
        "{:0>2}:{:0>2}:{:0>2}",
        remaining_time.hours, remaining_time.minutes, remaining_time.seconds
    );
    let lbl_remaining_time: Label = builder.object("lbl_remaining_time").unwrap();
    lbl_remaining_time.set_text(&remaining_time_str);
}

fn colorize_current_prayer_label(
    builder: &Builder,
    today_prayers: &PrayerTimesWithDate,
    next_prayer_index: Prayer,
) {
    let lbl_next_prayer: Label = builder.object("lbl_next_prayer").unwrap();

    // Name Labels
    let lbl_fajr: Label = builder.object("lbl_fajr").unwrap();
    let lbl_sunrise: Label = builder.object("lbl_sunrise").unwrap();
    let lbl_dhuhr: Label = builder.object("lbl_dhuhr").unwrap();
    let lbl_asr: Label = builder.object("lbl_asr").unwrap();
    let lbl_maghrib: Label = builder.object("lbl_maghrib").unwrap();
    let lbl_isha: Label = builder.object("lbl_isha").unwrap();

    // Time Labels
    let lbl_fajr_time: Label = builder.object("lbl_fajr_time").unwrap();
    let lbl_sunrise_time: Label = builder.object("lbl_sunrise_time").unwrap();
    let lbl_dhuhr_time: Label = builder.object("lbl_dhuhr_time").unwrap();
    let lbl_asr_time: Label = builder.object("lbl_asr_time").unwrap();
    let lbl_maghrib_time: Label = builder.object("lbl_maghrib_time").unwrap();
    let lbl_isha_time: Label = builder.object("lbl_isha_time").unwrap();

    lbl_fajr_time.set_text(&today_prayers.Imsak);
    lbl_sunrise_time.set_text(&today_prayers.Gunes);
    lbl_dhuhr_time.set_text(&today_prayers.Ogle);
    lbl_asr_time.set_text(&today_prayers.Ikindi);
    lbl_maghrib_time.set_text(&today_prayers.Aksam);
    lbl_isha_time.set_text(&today_prayers.Yatsi);

    // Remove Green Style
    lbl_fajr.remove_css_class("success");
    lbl_sunrise.remove_css_class("success");
    lbl_dhuhr.remove_css_class("success");
    lbl_asr.remove_css_class("success");
    lbl_maghrib.remove_css_class("success");
    lbl_isha.remove_css_class("success");

    lbl_fajr_time.remove_css_class("success");
    lbl_sunrise_time.remove_css_class("success");
    lbl_dhuhr_time.remove_css_class("success");
    lbl_asr_time.remove_css_class("success");
    lbl_maghrib_time.remove_css_class("success");
    lbl_isha_time.remove_css_class("success");

    // Update lbl_next_prayer
    let next_prayer_name = match next_prayer_index {
        Prayer::Sunrise => {
            lbl_sunrise.add_css_class("success");
            lbl_sunrise_time.add_css_class("success");

            tr("to Sunrise")
        }
        Prayer::Dhuhr => {
            lbl_dhuhr.add_css_class("success");
            lbl_dhuhr_time.add_css_class("success");

            tr("to Dhuhr")
        }
        Prayer::Asr => {
            lbl_asr.add_css_class("success");
            lbl_asr_time.add_css_class("success");

            tr("to Asr")
        }
        Prayer::Maghrib => {
            lbl_maghrib.add_css_class("success");
            lbl_maghrib_time.add_css_class("success");

            tr("to Maghrib")
        }
        Prayer::Isha => {
            lbl_isha.add_css_class("success");
            lbl_isha_time.add_css_class("success");

            tr("to Isha")
        }
        _ => {
            lbl_fajr.add_css_class("success");
            lbl_fajr_time.add_css_class("success");

            tr("to Fajr")
        }
    };
    lbl_next_prayer.set_text(next_prayer_name);
}
