use std::{process::Command, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::server_context::{EventHandler, RsbarContext, RsbarContextContent};

const MAX_BRIGHTNESS: f64 = 100.0;
const MIN_BRIGHTNESS: f64 = 0.0;

pub struct BrightnessContext {
    brightness:    f64,
    event_handler: Option<Arc<Mutex<EventHandler>>>,
}

#[async_trait]
impl RsbarContextContent for BrightnessContext {
    async fn init(&mut self, event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()> {
        self.event_handler = Some(event_handler);

        self.update().await?;

        Ok(())
    }

    async fn update(&mut self) -> tokio::io::Result<()> {
        let output = Command::new("brightnessctl").arg("-m").output().unwrap();

        let result_string = String::from_utf8_lossy(&output.stdout);
        let mut brightness_value_chars = result_string.split(',').collect::<Vec<&str>>()[3].chars();
        brightness_value_chars.next_back();
        
        let brightness_value = brightness_value_chars.as_str().parse::<f64>();
        
        match brightness_value {
            Ok(value) => self.brightness = value,
            Err(_)    => self.brightness = 0.0,
        };

        let events = self.event_handler.as_mut().unwrap().lock().await;
        
        events.trigger_event("brightness/brightness", &self.brightness.to_string()).await;

        Ok(())
    }

    fn call(&mut self, procedure: &str, args: &str) -> Option<String> {
        match procedure {
            "setBrightness" => self.set_brightness(args),
            _ => None,
        }
    }
}

impl BrightnessContext {
    pub fn new() -> (String, RsbarContext) {
        let new_context = Box::new(BrightnessContext {
            brightness:    0.0,
            event_handler: None,
        });

        ("brightness".to_string(), RsbarContext::new(new_context))
    }

    fn set_brightness(&mut self, args: &str) -> Option<String> {
        let parse_result = args.parse::<f64>();

        if !parse_result.is_ok() {
            return None;
        }

        let value = parse_result.unwrap();

        if value < MIN_BRIGHTNESS || value > MAX_BRIGHTNESS {
            return None;
        }

        self.brightness = value;

        let _ = Command::new("brightnessctl").arg("-q").arg("set").arg(format!("{}%", value * 100.0)).status();
        Some(String::from(args))
    }
}
