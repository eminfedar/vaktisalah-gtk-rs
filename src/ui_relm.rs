use chrono::Utc;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{adw, gtk, RelmWidgetExt, WidgetTemplate};

use rust_i18n::t;

#[relm4::widget_template(pub)]
impl WidgetTemplate for MainPageFooter {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_vexpand: true,
            set_valign: gtk::Align::Center,

            #[name = "lbl_prayer_time_name"]
            gtk::Label {
                set_css_classes: &["title-3"]
            },

            #[name = "lbl_prayer_time"]
            gtk::Label {
                set_css_classes: &["title-3", "success", "shadow"]
            }
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for MainPagePrayerTimes {
    view! {
        gtk::Box {
            set_spacing: 5,
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                #[name = "lbl_fajr"]
                gtk::Label {
                    set_label: t!("Fajr").as_ref(),
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "lbl_fajr_time"]
                gtk::Label {
                    set_label: "00:00",
                    set_halign: gtk::Align::End,
                },
            },

            gtk::Box {
                #[name = "lbl_sunrise"]
                gtk::Label {
                    set_label: t!("Sunrise").as_ref(),
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "lbl_sunrise_time"]
                gtk::Label {
                    set_label: "00:00",
                    set_halign: gtk::Align::End,
                },
            },

            gtk::Box {
                #[name = "lbl_dhuhr"]
                gtk::Label {
                    set_label: t!("Dhuhr").as_ref(),
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "lbl_dhuhr_time"]
                gtk::Label {
                    set_label: "00:00",
                    set_halign: gtk::Align::End,
                },
            },

            gtk::Box {
                #[name = "lbl_asr"]

                gtk::Label {
                    set_label: t!("Asr").as_ref(),
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "lbl_asr_time"]
                gtk::Label {
                    set_label: "00:00",
                    set_halign: gtk::Align::End,
                },
            },

            gtk::Box {
                #[name = "lbl_maghrib"]
                gtk::Label {
                    set_label: t!("Maghrib").as_ref(),
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "lbl_maghrib_time"]
                gtk::Label {
                    set_label: "00:00",
                    set_halign: gtk::Align::End,
                },
            },

            gtk::Box {
                #[name = "lbl_isha"]
                gtk::Label {
                    set_label: t!("Isha").as_ref(),
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "lbl_isha_time"]
                gtk::Label {
                    set_label: "00:00",
                    set_halign: gtk::Align::End,
                },
            },
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for SettingsPage {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 3,

            gtk::CenterBox {

                #[wrap(Some)]
                #[name = "btn_dark_mode"]
                set_start_widget = &gtk::Button {

                    #[wrap(Some)]
                    set_child = &gtk::Image {
                        set_icon_name: Some("night-light-symbolic"),
                    },
                },

                #[wrap(Some)]
                set_center_widget = &gtk::Label {
                  set_label: t!("Settings").as_ref(),
                  set_css_classes: &["title-4"],
                },

                #[wrap(Some)]
                #[name = "btn_go_main"]
                set_end_widget = &gtk::Button {
                    gtk::Image {
                        set_icon_name: Some("edit-undo-symbolic"),
                    },
                },
            },

            gtk::Separator {
                set_margin_top: 7,
                set_margin_bottom: 7,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 11,

                gtk::Box {
                  set_spacing: 7,
                  gtk::Label {
                    set_label: format!("{}:", t!("Country")).as_ref(),
                  },

                  #[name = "dd_country"]
                  gtk::DropDown {
                    set_hexpand: true,
                    set_enable_search: true,
                  }
                },

                gtk::Box {
                  set_spacing: 7,
                  gtk::Label {
                    set_label: format!("{}:", t!("City")).as_ref(),
                  },

                  #[name = "dd_city"]
                  gtk::DropDown {
                    set_hexpand: true,
                    set_enable_search: true,
                  }
                },

                gtk::Box {
                  set_spacing: 7,
                  gtk::Label {
                    set_label: format!("{}:", t!("District")).as_ref(),
                  },

                  #[name = "dd_district"]
                  gtk::DropDown {
                    set_hexpand: true,
                    set_enable_search: true,
                  }
                },

                gtk::Box {
                  set_spacing: 7,
                  gtk::Label {
                    set_label: format!("{}:", t!("Warn Min")).as_ref(),
                  },

                  #[name = "spn_warning_minutes"]
                  gtk::SpinButton {
                    set_adjustment: &gtk::Adjustment::builder()
                        .lower(1.0)
                        .upper(120.0)
                        .value(15.0)
                        .step_increment(1.0)
                        .page_increment(10.0)
                        .build()
                    ,
                    set_hexpand: true,
                  },
                },
            },

            gtk::Separator {
                set_margin_top: 7,
                set_margin_bottom: 7,
            },

            #[name = "btn_save"]
            gtk::Button {
                set_label: t!("Save").as_ref(),
                set_receives_default: true,
            },

            gtk::Label {
                set_markup: format!("<a href='https://github.com/eminfedar/vaktisalah-gtk-rs'><small>{}</small></a>", t!("SourceCodeIsOpen")).as_ref(),
                set_valign: gtk::Align::End,
                set_vexpand: true,
            },
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for MainPage {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 3,

            gtk::CenterBox {
                #[wrap(Some)]
                set_center_widget = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "lbl_district_name"]
                    gtk::Label {
                        set_label: t!("District").as_ref(),
                        set_css_classes: &["title-2"],
                    },
                    #[name = "lbl_city_name"]
                    gtk::Label {
                        set_label: t!("City").as_ref(),
                        set_css_classes: &["dim-label"],
                    },
                },

                #[wrap(Some)]
                #[name = "btn_go_settings"]
                set_end_widget = &gtk::Button {

                    #[wrap(Some)]
                    set_child = &gtk::Image {
                        set_icon_name: Some("emblem-system-symbolic"),
                    },
                    set_valign: gtk::Align::Start,
                    set_vexpand: true,
                },
            },

            gtk::Box {
                set_margin_top: 11,

                gtk::Label {
                  set_label: t!("Gregorian").as_ref(),
                  set_hexpand: true,
                  set_halign: gtk::Align::Start,
                },

                gtk::Label {
                  set_label: &Utc::now().format("%d %B %Y").to_string(),
                  set_halign: gtk::Align::End,
                }
            },
            gtk::Box {
                gtk::Label {
                  set_label: t!("Hijri").as_ref(),
                  set_css_classes: &["success"],
                  set_hexpand: true,
                  set_halign: gtk::Align::Start,
                },

                #[name = "lbl_date_hijri"]
                gtk::Label {
                  set_css_classes: &["success"],
                  set_halign: gtk::Align::End,
                },
            },

            gtk::Separator {
                set_margin_top: 7,
                set_margin_bottom: 7,
            },

            // == PRAYER TIMES == //

            #[template]
            #[name = "prayer_times"]
            MainPagePrayerTimes{},

            // ======= //


            gtk::Separator {
                set_margin_top: 7,
                set_margin_bottom: 7,
            },

            #[template]
            #[name = "remaining_time_footer"]
            MainPageFooter{}

        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for MainWindow {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::HeaderBar {
                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                    set_title: "Vakt-i Salah",
                }
            },

            #[name = "toast_overlay"]
            adw::ToastOverlay {
                #[wrap(Some)]
                #[name(stk_pages)]
                set_child = &gtk::Stack{
                    set_margin_all: 11,
                    set_margin_top: 7,

                    #[template]
                    #[name(main_page)]
                    add_child = &MainPage {} -> {
                        set_name: "main",
                    },

                    #[template]
                    #[name(settings_page)]
                    add_child = &SettingsPage {} -> {
                        set_name: "settings",
                    },

                }
            }

        }
    }
}
