use tray_item::TrayItem;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::error::Error;

pub enum TrayState {
    Running,
    Stopped,
}

pub struct TrayUi {
    tray: TrayItem,
    state: TrayState,
}

impl TrayUi {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut tray = TrayItem::new("Syncthingers", tray_item::IconSource::Resource("assets/icons/syncthing_red.ico"))?;
        // tray.set_tooltip("Syncthing is stopped").ok(); // set_tooltip does not exist, so this is removed
        Ok(Self {
            tray,
            state: TrayState::Stopped,
        })
    }

    pub fn set_state(&mut self, state: TrayState) {
        match state {
            TrayState::Running => {
                let _ = self.tray.set_icon(tray_item::IconSource::Resource("assets/icons/syncthing_green.ico"));
            }
            TrayState::Stopped => {
                let _ = self.tray.set_icon(tray_item::IconSource::Resource("assets/icons/syncthing_red.ico"));
            }
        }
        self.state = state;
    }

    pub fn add_menu<F>(&mut self, start_stop: F, open_web: F, open_config: F, exit: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let start_stop_label = match self.state {
            TrayState::Running => "Stop Syncthing",
            TrayState::Stopped => "Start Syncthing",
        };
        let _ = self.tray.add_menu_item(start_stop_label, start_stop);
        let _ = self.tray.add_menu_item("Open Syncthing Web UI", open_web);
        let _ = self.tray.add_menu_item("Open App Configuration", open_config);
        let _ = self.tray.add_menu_item("Exit", exit);
    }
}
