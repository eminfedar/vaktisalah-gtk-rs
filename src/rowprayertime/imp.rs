use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

#[derive(Default, gtk::CompositeTemplate, glib::Properties)]
#[properties(wrapper_type=super::RowPrayerTime)]
#[template(file = "ui/RowPrayerTime.blp")]
pub struct RowPrayerTime {
    #[property(get, set)]
    pub title: RefCell<String>,

    #[property(get, set)]
    pub value: RefCell<String>,

    #[property(get, set)]
    pub is_green: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for RowPrayerTime {
    const NAME: &'static str = "RowPrayerTime";
    type Type = super::RowPrayerTime;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for RowPrayerTime {}

impl WidgetImpl for RowPrayerTime {}
impl BoxImpl for RowPrayerTime {}
