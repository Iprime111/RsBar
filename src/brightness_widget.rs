use std::process::Command;

use crate::bar_widget::BarWidget;
use crate::slider_widget::SliderWidget;

use gtk4::prelude::BoxExt;

const MAX_BRIGHTNESS: f64 = 100.0;
const SLIDER_HEIGHT:  i32 = 100;
const ICON: [&str; 1] = ["󰖙"];

#[derive(Clone)]
pub struct BrightnessWidget {
    slider_widget: SliderWidget,
}

impl BrightnessWidget {
    pub fn new(duration: u32) -> Self {
        BrightnessWidget {
            slider_widget: SliderWidget::builder()
                .icons(&ICON.map(|x| x.to_string()))
                .transition_duration(duration)
                .slider_height(SLIDER_HEIGHT)
                .max_value(MAX_BRIGHTNESS)
                .set_value_callback(set_system_brightness)
                .get_value_callback(get_system_brightness)
                .button_class("brightness-widget-button")
                .slider_class("brightness-widget-slider")
                .container_class("brightness-widget-container")
                .label_class("brightness-widget-label")
                .main_class("brightness-widget")
                .build()
        }
    }
}

impl BarWidget for BrightnessWidget {
    fn update_widget(&mut self) {
        self.slider_widget.update_widget();
    }

    fn bind_widget(&self, container: &impl BoxExt) {
        self.slider_widget.bind_widget(container);
    }
}

fn set_system_brightness(brightness: f64) {
    let _ = Command::new("brightnessctl").arg("-q").arg("set").arg(format!("{}%", brightness * 100.0)).status();
}

fn get_system_brightness() -> f64 {
    let output = Command::new("brightnessctl").arg("-m").output().unwrap();

    let result_string = String::from_utf8_lossy(&output.stdout);
    let mut brightness_value_chars = result_string.split(',').collect::<Vec<&str>>()[3].chars();
    brightness_value_chars.next_back();

    let brightness_value = brightness_value_chars.as_str().parse::<f64>();

    match brightness_value {
        Ok(value) => value / 100.0,
        Err(_)    => -1.0,
    }
}

