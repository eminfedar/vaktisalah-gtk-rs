use gtk4::prelude::ButtonExt;
use gtk4::{Builder, Button, Label};
use simple_localization::tr;

macro_rules! translations_labels {
    ($builder:ident, $($widget:ident => $val:expr),* $(,)*) => {
        $(
            let $widget:Label = $builder.object(stringify!($widget)).unwrap();
            $widget.set_markup(tr($val));
        )*
    };
}

macro_rules! translations_buttons {
    ($builder:ident, $($widget:ident => $val:expr),* $(,)*) => {
        $(
            let $widget:Button = $builder.object(stringify!($widget)).unwrap();
            $widget.set_label(tr($val));
        )*
    };
}

pub fn translate_ui(builder: &Builder) {
    translations_labels! {
        builder,
        lbl_gregorian => "Gregorian",
        lbl_hijri => "Hijri",

        lbl_fajr => "Fajr",
        lbl_sunrise => "Sunrise",
        lbl_dhuhr => "Dhuhr",
        lbl_asr => "Asr",
        lbl_maghrib => "Maghrib",
        lbl_isha => "Isha",

        lbl_settings => "Settings",
        lbl_settings_country => "Country",
        lbl_settings_city => "City",
        lbl_settings_district => "District",
        lbl_warn_min => "Warn Min.",

        lbl_open_source => "<a href='github.com/eminfedar/vaktisalah-gtk-rs'><small>This project's source code is open</small></a>"
    };

    translations_buttons! {
        builder,

        btn_save => "Save"
    };
}
