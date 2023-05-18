use std::cell::Cell;

use glib::once_cell::sync::Lazy;
use glib::{ParamSpec, ParamSpecString, Value};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::ui::ListItemIDNameGtk;

// Object holding the state
#[derive(Default)]
pub struct ListItemIDName {
    item_id: Cell<String>,
    item_name: Cell<String>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ListItemIDName {
    const NAME: &'static str = "MyGtkAppListItemIDName";
    type Type = ListItemIDNameGtk;
}

// Trait shared by all GObjects
impl ObjectImpl for ListItemIDName {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecString::builder("itemId").build(),
                ParamSpecString::builder("itemName").build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "itemId" => {
                let v = value.get().expect("The value needs to be of type `i32`.");
                self.item_id.replace(v);
            }
            "itemName" => {
                let v = value.get().expect("The value needs to be of type `i32`.");
                self.item_name.replace(v);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
        unsafe {
            match pspec.name() {
                "itemId" => (*self.item_id.as_ptr()).to_value(),
                "itemName" => (*self.item_name.as_ptr()).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
