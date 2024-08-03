use core::str;
use std::process::Command;

use crate::server_context::{RsbarContext, RsbarContextContent};

pub struct VolumeContext {
    volume: f64,
    is_muted: bool,
}

impl RsbarContextContent for VolumeContext {
    fn init(&mut self) {
        self.volume   = 0.0;
        self.is_muted = false;
    }

    fn update(&mut self) {
        let output = Command::new("wpctl").arg("get-volume").arg("@DEFAULT_AUDIO_SINK@").output().unwrap();

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
    }

    fn query(&self, parameter: &str) -> Option<String> {
        match parameter {
            "volume"  => Some(self.volume.to_string()),
            "isMuted" => Some(self.is_muted.to_string()),
            _ => None,
        }
    }

    fn call(&mut self, procedure: &str, args: &str) -> Option<String> {
        match procedure {
            "setVolume" => self.set_volume(args),
            "setMuted"  => self.set_muted(args),
            _ => None,
        }
    }
}

impl VolumeContext {
    pub fn new() -> RsbarContext {
        let new_context = Box::new(VolumeContext {
            volume: 0.0,
            is_muted: false,
        });

        RsbarContext::new("volume", new_context)
    }

    fn set_volume(&self, args: &str) -> Option<String> {
        None
    }

    fn set_muted(&self, args: &str) -> Option<String> {
        None
    }
}
