use std::process::Command;

use crate::bar_widget::BarWidget;
use crate::slider_widget::SliderWidget;

use gtk4::prelude::BoxExt;

const MAX_VOLUME:    f64 = 100.0;
const SLIDER_HEIGHT: i32 = 100;
const VOLUME_ICONS: [&str; 4] = ["", "󰕿", "󰖀", "󰕾"];

#[derive(Clone)]
pub struct VolumeWidget {
    slider_widget: SliderWidget,
}

impl VolumeWidget {
    pub fn new(duration: u32) -> Self {
        VolumeWidget {
            slider_widget: SliderWidget::builder()
                .icons(&VOLUME_ICONS.map(|x| x.to_string()))
                .transition_duration(duration)
                .slider_height(SLIDER_HEIGHT)
                .max_value(MAX_VOLUME)
                .set_value_callback(set_system_volume)
                .get_value_callback(get_system_volume)
                .click_callback(toggle_mute)
                .button_class("volume-widget-button")
                .slider_class("volume-widget-slider")
                .container_class("volume-widget-container")
                .label_class("volume-widget-label")
                .main_class("volume-widget")
                .build()
        }
    }
}

impl BarWidget for VolumeWidget {
    fn update_widget(&mut self) {
        self.slider_widget.update_widget();
    }

    fn bind_widget(&self, container: &impl BoxExt) {
        self.slider_widget.bind_widget(container);
    }
}

fn toggle_mute() {
    let _ = Command::new("wpctl").arg("set-mute").arg("@DEFAULT_AUDIO_SINK@").arg("toggle").status();
}

fn set_system_volume(volume: f64) {
    if get_system_volume() < 0.0 {
        toggle_mute();
    }

    let _ = Command::new("wpctl").arg("set-volume").arg("@DEFAULT_AUDIO_SINK@").arg(format!("{}%", volume * 100.0)).status();
}

fn get_system_volume() -> f64 {
    let output = Command::new("wpctl").arg("get-volume").arg("@DEFAULT_AUDIO_SINK@").output().unwrap();

    let result_string = String::from_utf8_lossy(&output.stdout);
    let mut sound_value_chars = result_string.split_once(' ').unwrap().1.chars();
    sound_value_chars.next_back();

    let sound_value = sound_value_chars.as_str().parse::<f64>();

    match sound_value {
        Ok(value) => value,
        Err(_)    => -1.0,
    }
}
