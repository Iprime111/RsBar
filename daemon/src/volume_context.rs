use core::str;
use std::{io::ErrorKind, process::Command, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::rsbar_context::{EventHandler, RsbarContext, RsbarContextContent};

const MAX_VOLUME: f64 = 100.0;
const MIN_VOLUME: f64 = 0.0;

pub struct VolumeContext {
    volume:        f64,
    is_muted:      bool,
    event_handler: Option<Arc<Mutex<EventHandler>>>,
}

#[async_trait]
impl RsbarContextContent for VolumeContext {
    async fn init(&mut self, event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()>{
        self.event_handler = Some(event_handler);

        self.update().await?;

        Ok(())
    }

    async fn update(&mut self) -> tokio::io::Result<()> {
        let output = Command::new("wpctl").arg("get-volume").arg("@DEFAULT_AUDIO_SINK@").output()?;

        let result_string = String::from_utf8_lossy(&output.stdout);
        let mut sound_value_chars = result_string.split_once(' ').unwrap().1.chars();
        sound_value_chars.next_back();

        let sound_value = sound_value_chars.as_str().parse::<f64>();

        match sound_value {
            Ok(value) => {
                self.volume   = value;
                self.is_muted = false;
            },
            Err(_) => self.is_muted = true,
        };

        self.force_events().await?;

        Ok(())
    }

    async fn call(&mut self, procedure: &str, args: &str) -> tokio::io::Result<()> {
        match procedure {
            "setVolume"  => self.set_volume(args)?,
            "toggleMute" => self.toggle_muted(args)?,
            _ => return Err(std::io::Error::new(ErrorKind::NotFound, format!("Bad procedure value for volume context: {procedure}"))),
        };

        self.force_events().await?; // TODO use update instead of force_events?

        Ok(())
    }

    async fn force_events(&mut self) -> tokio::io::Result<()> {
        if self.event_handler.is_none() {
            return Err(std::io::Error::new(ErrorKind::NotFound, "Event handler was not found"));
        }

        let events = self.event_handler.as_mut().unwrap().lock().await;

        events.trigger_event("volume/volume",  &self.volume.to_string()).await;
        events.trigger_event("volume/isMuted", &self.is_muted.to_string()).await;

        Ok(())
    }
}

impl VolumeContext {
    pub fn new() -> (String, RsbarContext) {
        let new_context = Box::new(VolumeContext {
            volume:        0.0,
            is_muted:      false,
            event_handler: None,
        });

        ("volume".to_string(), RsbarContext::new(new_context))
    }

    fn set_volume(&mut self, args: &str) -> tokio::io::Result<()> {
        let parse_result = args.parse::<f64>();

        if parse_result.is_err() {
            return Err(std::io::Error::new(ErrorKind::Other, format!("Bad volume value: {args}")));
        }

        let value = parse_result.unwrap();

        if value < MIN_VOLUME || value > MAX_VOLUME {
            return Err(std::io::Error::new(ErrorKind::Other, format!("Volume value is out of range: {args}")));
        }

        self.volume = value;   

        Command::new("wpctl").arg("set-volume").arg("@DEFAULT_AUDIO_SINK@").arg(format!("{}%", value * 100.0)).status()?;
        
        Ok(())
    }

    fn toggle_muted(&mut self, _args: &str) -> tokio::io::Result<()> {
        self.is_muted = !self.is_muted;
        Command::new("wpctl").arg("set-mute").arg("@DEFAULT_AUDIO_SINK@").arg("toggle").status()?;

        Ok(())
    }
}
