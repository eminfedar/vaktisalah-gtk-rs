mod imp;

use gtk::glib;

glib::wrapper! {
    pub struct RowPrayerTime(ObjectSubclass<imp::RowPrayerTime>)
    @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget;
}
