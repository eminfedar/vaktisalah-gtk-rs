use async_channel::Sender;
use gettextrs::gettext;
use gtk::glib;

use crate::TrayMessage;

pub struct MyTray {
    pub sender: Sender<TrayMessage>,
}

impl std::fmt::Debug for MyTray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyTray")
    }
}

impl ksni::Tray for MyTray {
    fn icon_name(&self) -> String {
        "io.github.eminfedar.vaktisalah-gtk-rs".into()
    }
    fn title(&self) -> String {
        "VaktiSalah".into()
    }
    // NOTE: On some system trays, `id` is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        println!("Tray icon activated");

        let sender = self.sender.clone();

        glib::spawn_future(async move {
            sender.send(TrayMessage::Activate).await.unwrap();
        });
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: gettext("Show").into(),
                icon_name: "view-fullscreen-symbolic".into(),
                activate: Box::new(|this: &mut Self| {
                    println!("Tray::Show");

                    let sender = this.sender.clone();

                    glib::spawn_future(async move {
                        sender.send(TrayMessage::Activate).await.unwrap();
                    });
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Exit").into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(|this: &mut Self| {
                    println!("Tray::Exit");

                    let sender = this.sender.clone();

                    glib::spawn_future(async move {
                        sender.send(TrayMessage::Exit).await.unwrap();
                    });
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
