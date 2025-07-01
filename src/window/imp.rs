use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use adw::prelude::*;
use adw::subclass::prelude::*;
use async_channel::Sender;
use gtk::glib;

use gtk::StringList;

use crate::prayer::PrayerTimesWithDate;
use crate::preferences::PreferencesJson;
use crate::rowprayertime::RowPrayerTime;

#[derive(Debug)]
pub enum Message {
    CityListArrived(Result<HashMap<String, String>, reqwest::Error>, String),
    DistrictListArrived(Result<HashMap<String, String>, reqwest::Error>, String),

    PrayerTimesArrived(Result<Vec<PrayerTimesWithDate>, reqwest::Error>),
}

#[derive(Default, gtk::CompositeTemplate, glib::Properties)]
#[properties(wrapper_type=super::MainWindow)]
#[template(file = "ui/MainWindow.blp")]
pub struct MainWindow {
    #[template_child]
    pub toast_overlay: TemplateChild<adw::ToastOverlay>,

    #[template_child]
    pub dummy_row: TemplateChild<RowPrayerTime>,

    #[template_child]
    pub navigation_view: TemplateChild<adw::NavigationView>,

    #[template_child]
    pub main_page: TemplateChild<adw::NavigationPage>,

    pub preferences: RefCell<PreferencesJson>,

    // Prayer times
    pub todays_prayers: RefCell<Option<PrayerTimesWithDate>>,
    pub tomorrows_prayers: RefCell<Option<PrayerTimesWithDate>>,
    pub visible_day: RefCell<i8>,

    pub sender: RefCell<Option<Sender<Message>>>,

    // Date
    #[property(get, set)]
    pub gregorian_date: RefCell<String>,
    #[property(get, set)]
    pub hijri_date: RefCell<String>,

    // Times
    #[property(get, set)]
    pub time_fajr: RefCell<String>,
    #[property(get, set)]
    pub time_sunrise: RefCell<String>,
    #[property(get, set)]
    pub time_dhuhr: RefCell<String>,
    #[property(get, set)]
    pub time_asr: RefCell<String>,
    #[property(get, set)]
    pub time_maghrib: RefCell<String>,
    #[property(get, set)]
    pub time_isha: RefCell<String>,

    // Remaining Time
    #[property(get, set)]
    pub next_prayer_name: RefCell<String>,
    #[property(get, set)]
    pub next_prayer_time: RefCell<String>,

    #[property(get, set)]
    pub current_prayer: Cell<i32>,

    // Settings:
    #[property(get, set)]
    pub warn_min: Cell<f64>,

    // Models
    #[property(get, set)]
    pub model_country: RefCell<StringList>,
    #[property(get, set)]
    pub model_city: RefCell<StringList>,
    #[property(get, set)]
    pub model_district: RefCell<StringList>,

    pub countries: RefCell<HashMap<String, String>>,
    pub countries_en: RefCell<HashMap<String, String>>,
    pub cities: RefCell<HashMap<String, String>>,
    pub districts: RefCell<HashMap<String, String>>,

    pub country: RefCell<String>,
    pub city: RefCell<String>,
    pub district: RefCell<String>,

    #[property(get, set)]
    pub district_title: RefCell<String>,

    #[property(get, set)]
    pub selected_country_index: Cell<i32>,
    #[property(get, set)]
    pub selected_city_index: Cell<i32>,
    #[property(get, set)]
    pub selected_district_index: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for MainWindow {
    const NAME: &'static str = "MainWindow";
    type Type = super::MainWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_instance_callbacks();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for MainWindow {}

impl WidgetImpl for MainWindow {}
impl WindowImpl for MainWindow {}
impl ApplicationWindowImpl for MainWindow {}
impl AdwApplicationWindowImpl for MainWindow {}
