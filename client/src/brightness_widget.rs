use crate::bar_widget::BarWidget;
use crate::slider_widget::SliderWidget;
use crate::unix_sockets::ChannelsData;

const MAX_BRIGHTNESS: f64 = 100.0;
const SLIDER_HEIGHT:  i32 = 100;
const ICON: [&str; 1] = ["󰖙"];

const EVENTS_LIST: &[&str] = &[
    "brightness/brightness",
];

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
                .slider_class("brightness-widget-slider")
                .container_class("brightness-widget-container")
                .label_class("brightness-widget-label")
                .main_class("brightness-widget")
                .build()
        }
    }
}

impl BarWidget for BrightnessWidget {
    fn bind_widget(&self, container: &gtk4::Box) {
        self.slider_widget.bind_widget(container);
    }

    fn events_list(&self) -> &'static[&'static str] {
        EVENTS_LIST
    }

    fn bind_channels(&self, channels_data: ChannelsData) {
        self.slider_widget.bind_channels(channels_data);
    }
}

fn set_system_brightness(brightness: f64) -> String {
    format!("brightness/setBrightness/{}", brightness)
}

fn get_system_brightness(name: &str, value: &str) -> Option<f64> {

    if name != "brightness/brightness" {
        return None;
    }

    let value_float = value.parse::<f64>();

    if value_float.is_err() {
        return None;
    }

    Some(value_float.unwrap())
}

