use crate::CommandMessage;
use relm4::Sender;
use rust_i18n::t;

pub struct MyTray {
    pub sender: Sender<CommandMessage>,
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
        self.sender.send(CommandMessage::Show).unwrap_or(());
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: t!("Show").into(),
                icon_name: "view-fullscreen-symbolic".into(),
                activate: Box::new(|this: &mut Self| this.sender.send(CommandMessage::Show).unwrap_or(())),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: t!("Exit").into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(|this: &mut Self| this.sender.send(CommandMessage::Exit).unwrap_or(())),
                ..Default::default()
            }
            .into(),
        ]
    }
}
