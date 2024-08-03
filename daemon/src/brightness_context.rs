use std::process::Command;

use crate::server_context::{RsbarContext, RsbarContextContent};

pub struct BrightnessContext {
    brightness: f64,
}

impl RsbarContextContent for BrightnessContext {
    fn init(&mut self) {
        todo!()
    }

    fn update(&mut self) {
        let output = Command::new("brightnessctl").arg("-m").output().unwrap();

        let result_string = String::from_utf8_lossy(&output.stdout);
        let mut brightness_value_chars = result_string.split(',').collect::<Vec<&str>>()[3].chars();
        brightness_value_chars.next_back();
        
        let brightness_value = brightness_value_chars.as_str().parse::<f64>();
        
        match brightness_value {
            Ok(value) => self.brightness = value / 100.0,
            Err(_)    => self.brightness = 0.0,
        };
    }

    fn query(&self, parameter: &str) -> Option<String> {
        match parameter {
            "brightness" => Some(self.brightness.to_string()),
            _ => None,
        }
    }

    fn call(&mut self, procedure: &str, args: &str) -> Option<String> {
        match procedure {
            "setBrightness" => self.set_brightness(args),
            _ => None,
        }
    }
}

impl BrightnessContext {
    fn new() -> RsbarContext {
        let new_context = Box::new(BrightnessContext {
            brightness: 0.0,
        });

        RsbarContext::new("brightness", new_context)
    }

    fn set_brightness(&self, args: &str) -> Option<String> {
        None
    }
}
