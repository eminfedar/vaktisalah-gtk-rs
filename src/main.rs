use adw::Application;
use gtk::prelude::*;
use gtk4 as gtk;

use networking::is_prayer_times_valid;
use preferences::read_preferences_json_file;
use ui::build_ui;

use crate::networking::update_prayer_times_on_network;
use crate::preferences::save_preferences_json;

mod listitem;
mod networking;
mod prayer;
mod preferences;
mod sound;
mod translations;
mod ui;

fn on_activate(application: &Application) {
    if application.windows().len() == 1 {
        application.windows().first().as_ref().unwrap().present();
        return;
    }

    // Read preferences
    let mut preferences_json = match read_preferences_json_file() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Failed to read preferences.json: {err:?}");
            return;
        }
    };

    // Check if prayer times up-to-date. Update if it isn't.
    if !is_prayer_times_valid(&preferences_json) {
        // Update monthly prayer times from network
        eprintln!("Prayer Times are out of date. Updating...");
        match update_prayer_times_on_network(&mut preferences_json) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Failed to upgrade Prayer Times from internet: {err:?}");
                return;
            }
        }

        // Save the new prayer times to preferences.json
        match save_preferences_json(&preferences_json) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Failed to save preferences.json: {err:?}");
                return;
            }
        }
    }

    build_ui(application, preferences_json);
}

fn main() {
    let application = Application::builder()
        .application_id("io.github.eminfedar.vaktisalah-gtk-rs")
        .build();

    application.connect_activate(on_activate);

    application.run();
}
