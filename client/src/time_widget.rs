use crate::{bar_widget::BarWidget, unix_sockets::ChannelsData};
use gtk4::{glib::{clone::Downgrade, MainContext}, prelude::{BoxExt, WidgetExt}};

const EVENTS_LIST: &[&str] = &[
    "time/time",
];

pub struct TimeWidget {
    label: gtk4::Label,
}

impl TimeWidget {
    pub fn new() -> TimeWidget {
        let time_widget = gtk4::Label::new(Some(""));

        time_widget.set_justify(gtk4::Justification::Center);
        time_widget.add_css_class("time-widget");

        TimeWidget {label: time_widget}
    }
}

impl BarWidget for TimeWidget {
    fn bind_widget(&self, container: &gtk4::Box) {
        container.append(&self.label);
    }

    fn events_list(&self) -> &'static[&'static str] {
        EVENTS_LIST
    }
    
    fn bind_channels(&self, mut channels_data: ChannelsData) {
        let weak_label = self.label.downgrade();

        MainContext::default().spawn_local(async move {

            while let Ok(event) = channels_data.event_rx.recv().await {
                if event.name != EVENTS_LIST[0] {
                    continue;
                }

                weak_label.upgrade().unwrap().set_text(&event.value);
            }
        });
    }
}
