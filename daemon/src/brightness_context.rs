use std::{io::ErrorKind, process::Command, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::rsbar_context::{EventHandler, RsbarContext, RsbarContextContent};

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
        let output = Command::new("brightnessctl").arg("-m").output()?;

        let result_string = String::from_utf8_lossy(&output.stdout);
        let mut brightness_value_chars = result_string.split(',').collect::<Vec<&str>>()[3].chars();
        brightness_value_chars.next_back();
        
        let brightness_value = brightness_value_chars.as_str().parse::<f64>();
        
        match brightness_value {
            Ok(value) => self.brightness = value / 100.0,
            Err(_)    => self.brightness = 0.0,
        };

        self.force_events().await?;

        Ok(())
    }

    async fn call(&mut self, procedure: &str, args: &str) -> tokio::io::Result<()> {
        match procedure {
            "setBrightness" => self.set_brightness(args)?,
            _ => return Err(std::io::Error::new(ErrorKind::NotFound, format!("Bad procedure value for brightness context: {procedure}"))),
        };

        self.force_events().await?;

        Ok(())
    }

    async fn force_events(&mut self) -> tokio::io::Result<()> {
        if self.event_handler.is_none() {
            return Err(std::io::Error::new(ErrorKind::NotFound, "Event handler was not found"));
        }

        self.event_handler.as_mut().unwrap().lock().await
            .trigger_event("brightness/brightness", &self.brightness.to_string()).await;

        Ok(())
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

    fn set_brightness(&mut self, args: &str) -> tokio::io::Result<()> {
        let parse_result = args.parse::<f64>();

        if !parse_result.is_ok() {
            return Err(std::io::Error::new(ErrorKind::Other, format!("Bad brightness value: {args}")));
        }

        let value = parse_result.unwrap();

        if value < MIN_BRIGHTNESS || value > MAX_BRIGHTNESS {
            return Err(std::io::Error::new(ErrorKind::Other, format!("Brightness value is out of range: {args}")));
        }

        self.brightness = value;

        Command::new("brightnessctl").arg("-q").arg("set").arg(format!("{}%", value * 100.0)).status()?;

        Ok(())
    }
}
