use std::{io::ErrorKind, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use log::info;
use regex::Regex;
use tokio::{io::{AsyncReadExt, AsyncSeekExt}, sync::Mutex};

use crate::rsbar_context::{EventHandler, RsbarContext, RsbarContextContent};

const BATTERY_SYMLINK_PATH: &str = "/sys/class/power_supply/";
const BATTERY_DIR_REGEX: &str = "^BAT[0-9]+$";

pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

pub struct BatteryContext {
    capacity:       u32,
    status:         BatteryStatus,
    capacity_file:  Option<tokio::fs::File>,
    status_file:    Option<tokio::fs::File>,
    event_handler:  Option<Arc<Mutex<EventHandler>>>,
}

#[async_trait]
impl RsbarContextContent for BatteryContext {
    async fn init(&mut self, event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()> {
        self.event_handler = Some(event_handler);

        let battery_dir = find_battery_dir().await?;

        info!("Battery info dir: {}", battery_dir.to_string_lossy());

        self.capacity_file = Some(tokio::fs::File::open(battery_dir.join("capacity")).await?);
        self.status_file   = Some(tokio::fs::File::open(battery_dir.join("status")).await?);

        self.update().await?;

        Ok(())
    }

    async fn update(&mut self) -> tokio::io::Result<()> {
    
        if self.capacity_file.is_none() {
            return Err(std::io::Error::new(ErrorKind::NotFound, "Battery symlink is closed"));
        }

        if self.status_file.is_none() {
            return Err(std::io::Error::new(ErrorKind::NotFound, "Battery symlink is closed"));
        }
    
        match read_file_content(self.capacity_file.as_mut().unwrap()).await?.parse::<u32>() {
            Ok(capacity) => self.capacity = capacity,
            Err(err) => return Err(std::io::Error::new(ErrorKind::NotFound, format!("Bad capacity value: {}", err.to_string()))),
        }

        let status_string = read_file_content(self.status_file.as_mut().unwrap()).await?;

        match status_string.as_ref() {
            "Charging"     => self.status = BatteryStatus::Charging,
            "Discharging"  => self.status = BatteryStatus::Discharging,
            "Full"         => self.status = BatteryStatus::Full,
            "Not charging" => self.status = BatteryStatus::NotCharging,
            "Unknown"      => self.status = BatteryStatus::Unknown,
            _ => return Err(std::io::Error::new(ErrorKind::NotFound, format!("Bad status value: {status_string}"))),
        }

        self.force_events().await?;

        Ok(())
    }

    async fn call(&mut self, _procedure: &str, _args: &str) -> tokio::io::Result<()> {
        Err(std::io::Error::new(ErrorKind::NotFound, "Battery context does not support calls"))
    }

    async fn force_events(&mut self) -> tokio::io::Result<()> {
        if self.event_handler.is_none() {
            return Err(std::io::Error::new(ErrorKind::NotFound, "Event handler was not found"));
        }

        let events = self.event_handler.as_mut().unwrap().lock().await;

        events.trigger_event("battery/capacity", &self.capacity.to_string()).await;
        events.trigger_event("battery/status",   &self.status.to_string()).await;

        Ok(())
    }
}

impl std::fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryStatus::Charging    => write!(f, "Charging"),
            BatteryStatus::Discharging => write!(f, "Discharging"),
            BatteryStatus::Full        => write!(f, "Full"),
            BatteryStatus::NotCharging => write!(f, "NotCharging"),
            BatteryStatus::Unknown     => write!(f, "Unknown"),
        }
    }
}

impl BatteryContext {
    pub fn new() -> (String, RsbarContext) {
        let new_context = Box::new(BatteryContext {
            capacity:       0,
            status:         BatteryStatus::Full,
            capacity_file:  None,
            status_file:    None,
            event_handler:  None,
        });

        ("battery".to_string(), RsbarContext::new(new_context))
    }
}

async fn read_file_content(file: &mut tokio::fs::File) -> tokio::io::Result<String> {

    let mut file_content = String::new();
    file.read_to_string(&mut file_content).await?;
    file.seek(std::io::SeekFrom::Start(0)).await?;

    Ok(file_content.trim().to_string())
}

async fn find_battery_dir() -> tokio::io::Result<std::path::PathBuf> {
    let battery_dir_regex_wrapped = Regex::new(BATTERY_DIR_REGEX);

    if battery_dir_regex_wrapped.is_err() {
        return Err(std::io::Error::new(ErrorKind::NotFound, format!("Invalid battery dir regex: {BATTERY_DIR_REGEX}")));
    }

    let battery_dir_regex = battery_dir_regex_wrapped.unwrap();

    let mut battery_dir_path: Option<PathBuf> = None;

    let mut paths = tokio::fs::read_dir(std::path::Path::new(BATTERY_SYMLINK_PATH)).await?;

    while let Ok(Some(entry)) = paths.next_entry().await {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        if let Some(dir_name) = path.file_name() {
            if let Some(dir_name_str) = dir_name.to_str() {
                if battery_dir_regex.is_match(dir_name_str) {
                    battery_dir_path = Some(path);
                }
            }
        }

    }

    match battery_dir_path {
        Some(dir) => Ok(dir),
        None      => Err(std::io::Error::new(ErrorKind::NotFound, "Event handler was not found")),
    }
}
