use crate::bar_widget::BarWidget;
use gtk4::prelude::{BoxExt, WidgetExt};
use chrono::Local;

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
    fn bind_widget(&self, container: &impl BoxExt) {
        container.append(&self.label);
    }

    fn update_widget(&mut self) {
        let time = current_time();

        self.label.set_text(&time);
    }
    
}

fn current_time() -> String {
    format!("{}", Local::now().format("%H\n%M"))
}
