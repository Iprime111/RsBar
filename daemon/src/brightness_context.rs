use std::{io::ErrorKind, sync::Arc};

use async_trait::async_trait;
use brightness::Brightness;
use futures::TryStreamExt;
use tokio::sync::Mutex;

use crate::rsbar_context::{EventHandler, RsbarContext, RsbarContextContent};

const MAX_BRIGHTNESS: u32 = 100;
const MIN_BRIGHTNESS: u32 = 0;

pub struct BrightnessContext {
    brightness:    u32,
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
 
        self.brightness = match get_brightness().await {
            Ok(value) => value,
            Err(err) => return Err(std::io::Error::new(ErrorKind::NotFound, err)),
        };
       
        self.force_events().await?;

        Ok(())
    }

    async fn call(&mut self, procedure: &str, args: &str) -> tokio::io::Result<()> {
        match procedure {
            "setBrightness" => {
                self.brightness = BrightnessContext::parse_brightness(args)?;
                if let Err(err) = set_brightness(self.brightness).await {
                    return Err(std::io::Error::new(ErrorKind::Other, format!("Unable to set the brightness value: {err}")));
                }
            },
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
            brightness:    0,
            event_handler: None,
        });

        ("brightness".to_string(), RsbarContext::new(new_context))
    }

    fn parse_brightness(args: &str) -> tokio::io::Result<u32> {

        let parse_result = args.parse::<u32>();
        
        if !parse_result.is_ok() {
            return Err(std::io::Error::new(ErrorKind::Other, format!("Bad brightness value: {args}")));
        }
        
        let value = parse_result.unwrap();
        
        if value < MIN_BRIGHTNESS || value > MAX_BRIGHTNESS {
            return Err(std::io::Error::new(ErrorKind::Other, format!("Brightness value is out of range: {args}")));
        }

        Ok(value)
    }
}

 async fn get_brightness() -> Result<u32, brightness::Error> {
    let brightness_device = brightness::brightness_devices().try_next().await?;

    match brightness_device {
        Some(device) => Ok(device.get().await?),
        None => Err(brightness::Error::ListingDevicesFailed(Box::new(std::io::Error::new(ErrorKind::NotFound, "Brightness device not found")))),
    }
}

async fn set_brightness(value: u32) -> Result<(), brightness::Error> {

    brightness::brightness_devices().try_for_each(|mut device| async move {
        device.set(value).await?;

        Ok(())
    }).await
}

