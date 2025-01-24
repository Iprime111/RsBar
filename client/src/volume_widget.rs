use crate::bar_widget::BarWidget;
use crate::slider_widget::{SliderFetchResult, SliderWidget};
use crate::unix_sockets::ChannelsData;

const MAX_VOLUME:    f64 = 100.0;
const SLIDER_HEIGHT: i32 = 100;
const VOLUME_ICONS: [&str; 4] = ["󰖁", "󰕿", "󰖀", "󰕾"];

const EVENTS_LIST: &[&str] = &[
    "volume/volume",
    "volume/isMuted",
];

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
                .slider_class("volume-widget-slider")
                .container_class("volume-widget-container")
                .label_class("volume-widget-label")
                .main_class("volume-widget")
                .build()
        }
    }
}

impl BarWidget for VolumeWidget {
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

fn toggle_mute() -> String {
    "volume/toggleMute/".to_string()
}

fn set_system_volume(volume: f64) -> String {
    format!("volume/setVolume/{}", (volume * MAX_VOLUME) as u32)
}

fn get_system_volume(name: &str, value: &str) -> SliderFetchResult {
    if name == EVENTS_LIST[0] {
        let value_float = value.parse::<f64>();
        
        if value_float.is_err() {
            return SliderFetchResult::None;
        }
        
        return SliderFetchResult::Value(value_float.unwrap() / MAX_VOLUME);
    } else if name == EVENTS_LIST[1] {
        return match value {
            "false" => SliderFetchResult::On,
            "true"  => SliderFetchResult::Off,
            _       => SliderFetchResult::None,
        };
    }

    SliderFetchResult::None
}
