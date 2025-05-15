use crate::error::{AppError, AppResult};
use log::{debug, info, error};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use tray_item::{TrayItem, IconSource};

// Menu actions
#[derive(Debug, Clone, Copy)]
pub enum TrayAction {
    StartSyncthing,
    StopSyncthing,
    OpenWebUI,
    OpenConfig,
    Exit,
}

// Path to icon resources
const ICON_STOPPED: &str = "resources/icons/syncthing_stopped.ico";
const ICON_RUNNING: &str = "resources/icons/syncthing_running.ico";
const APP_ICON: &str = "resources/icons/syncthingers.ico";

pub struct TrayManager {
    tray: TrayItem,
    action_sender: Sender<TrayAction>,
    current_tooltip: String,
}

impl TrayManager {
    pub fn new() -> AppResult<(Self, Sender<TrayAction>)> {
        debug!("Initializing system tray manager");
        
        // Create a channel for tray menu actions
        let (tx, rx) = channel();
        let action_sender = tx.clone();
        
        // Create tray icon with default icon
        let mut tray = TrayItem::new("Syncthingers", IconSource::Resource(ICON_STOPPED))
            .map_err(|e| AppError::Tray(format!("Failed to create tray icon: {}", e)))?;
            
        // Set up basic menu items
        Self::setup_tray_menu(&mut tray, tx.clone())?;
        
        // Create and return the tray manager
        let manager = TrayManager {
            tray,
            action_sender: tx,
            current_tooltip: "Syncthing is not running".to_string(),
        };
        
        Ok((manager, action_sender))
    }
    
    fn setup_tray_menu(tray: &mut TrayItem, tx: Sender<TrayAction>) -> AppResult<()> {
        info!("Setting up system tray menu");
        
        // Start Syncthing
        let start_tx = tx.clone();
        tray.add_menu_item("Start Syncthing", move || {
            if let Err(e) = start_tx.send(TrayAction::StartSyncthing) {
                error!("Failed to send Start action: {}", e);
            }
        })
        .map_err(|e| AppError::Tray(format!("Failed to add Start menu item: {}", e)))?;
        
        // Stop Syncthing
        let stop_tx = tx.clone();
        tray.add_menu_item("Stop Syncthing", move || {
            if let Err(e) = stop_tx.send(TrayAction::StopSyncthing) {
                error!("Failed to send Stop action: {}", e);
            }
        })
        .map_err(|e| AppError::Tray(format!("Failed to add Stop menu item: {}", e)))?;
        
        // Open Web UI
        let web_tx = tx.clone();
        tray.add_menu_item("Open Web UI", move || {
            if let Err(e) = web_tx.send(TrayAction::OpenWebUI) {
                error!("Failed to send Open Web UI action: {}", e);
            }
        })
        .map_err(|e| AppError::Tray(format!("Failed to add Open Web UI menu item: {}", e)))?;
        
        // Open Configuration
        let config_tx = tx.clone();
        tray.add_menu_item("Open Configuration", move || {
            if let Err(e) = config_tx.send(TrayAction::OpenConfig) {
                error!("Failed to send Open Config action: {}", e);
            }
        })
        .map_err(|e| AppError::Tray(format!("Failed to add Open Config menu item: {}", e)))?;
        
        // Exit
        let exit_tx = tx.clone();
        tray.add_menu_item("Exit", move || {
            if let Err(e) = exit_tx.send(TrayAction::Exit) {
                error!("Failed to send Exit action: {}", e);
            }
        })
        .map_err(|e| AppError::Tray(format!("Failed to add Exit menu item: {}", e)))?;
        
        debug!("Tray menu setup complete");
        Ok(())
    }
    
    pub fn update_icon(&mut self, syncthing_running: bool) -> AppResult<()> {
        let icon_path = if syncthing_running {
            ICON_RUNNING
        } else {
            ICON_STOPPED
        };
        
        debug!("Updating system tray icon to '{}', syncthing running: {}", icon_path, syncthing_running);
        
        self.tray.set_icon(IconSource::Resource(icon_path))
            .map_err(|e| AppError::Tray(format!("Failed to update tray icon: {}", e)))?;
            
        Ok(())
    }
    
    pub fn update_tooltip(&mut self, status: &str) -> AppResult<()> {
        debug!("Updating tooltip: {}", status);
        
        // Store tooltip text for reference
        self.current_tooltip = status.to_string();
        
        // In tray-item 0.10.0, there's no set_tooltip or new_with_tooltip method
        // We'll add the status to the title as a workaround
        let title = format!("Syncthingers - {}", status);
        
        // Create a new tray with the updated title that acts as our tooltip
        let current_icon = if self.current_tooltip.contains("running") {
            ICON_RUNNING
        } else {
            ICON_STOPPED
        };
        
        let mut new_tray = TrayItem::new(&title, IconSource::Resource(current_icon))
            .map_err(|e| AppError::Tray(format!("Failed to update tray tooltip: {}", e)))?;
        
        // Re-setup the menu
        Self::setup_tray_menu(&mut new_tray, self.action_sender.clone())?;
        
        // Replace the old tray
        self.tray = new_tray;
            
        Ok(())
    }
} 