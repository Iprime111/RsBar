use std::rc::Rc;

use gtk4::{glib::object::ObjectExt, prelude::GestureExt};
use gtk4::glib::SignalHandlerId;
use crate::bar_widget::BarWidget;
use gtk4::prelude::{BoxExt, WidgetExt, RangeExt};

const EPS: f64 = 1e-5;

#[derive(Clone)]
pub struct SliderWidget {
    slider:           gtk4::Scale,
    label:            gtk4::Label,
    container:        gtk4::Box,
    value_changed_signal: Rc<SignalHandlerId>,
    max_value:            f64,
    icons:                Vec<String>,
    get_value:            fn() -> f64,
    click:                fn(),
}

pub struct SliderWidgetBuilder {
    max_value:           f64,
    icons:               Vec<String>,
    slider_height:       i32,
    transition_duration: u32,
    set_value:           fn(f64),
    get_value:           fn() -> f64,
    click:               fn(),
    slider_class:        String,
    container_class:     String,
    label_class:         String,
    main_class:          String,
}

fn dummy_set(_: f64) {}
fn dummy_get() -> f64 {0.0}
fn dummy_click() {}

impl Default for SliderWidgetBuilder {
    fn default() -> Self {
        Self {
            max_value:           100.0,
            icons:               vec![],
            slider_height:       100,
            transition_duration: 1000,
            set_value:           dummy_set,
            get_value:           dummy_get,
            click:               dummy_click,
            slider_class:        "slider-widget-slider".to_string(),
            container_class:     "slider-widget-container".to_string(),
            label_class:         "slider-widget-label".to_string(),
            main_class:          "slider-widget".to_string(),
        }
    }
}

//TODO macros for builder functions
impl SliderWidgetBuilder {
    pub fn build(&self) -> SliderWidget {
        SliderWidget::from_builder(self)
    }

    pub fn max_value(&mut self, max_value: f64) -> &mut Self {
        self.max_value = max_value;
        self
    }

    pub fn icons(&mut self, icons :&[String]) -> &mut Self {
        self.icons = icons.to_vec();
        self
    }

    pub fn slider_height(&mut self, slider_height: i32) -> &mut Self {
        self.slider_height = slider_height;
        self
    }

    pub fn transition_duration(&mut self, transition_duration: u32) -> &mut Self {
        self.transition_duration = transition_duration;
        self
    }

    pub fn set_value_callback(&mut self, callback: fn(f64)) -> &mut Self {
        self.set_value = callback;
        self
    }

    pub fn get_value_callback(&mut self, callback: fn() -> f64) -> &mut Self {
        self.get_value = callback;
        self
    }

    pub fn click_callback(&mut self, callback: fn()) -> &mut Self {
        self.click = callback;
        self
    }

    pub fn slider_class(&mut self, slider_class: &str) -> &mut Self {
        self.slider_class = slider_class.to_string();
        self
    }

    pub fn container_class(&mut self, container_class: &str) -> &mut Self {
        self.container_class = container_class.to_string();
        self
    }

    pub fn label_class(&mut self, label_class: &str) -> &mut Self {
        self.label_class = label_class.to_string();
        self
    }

    pub fn main_class(&mut self, main_class: &str) -> &mut Self {
        self.main_class = main_class.to_string();
        self
    }
}

impl SliderWidget {
    pub fn builder() -> SliderWidgetBuilder {
        SliderWidgetBuilder::default()
    }
    
    pub fn new() -> Self {
        Self::from_builder(&SliderWidgetBuilder::default())
    }

    //TODO fix this BIG shit below
    fn from_builder(builder: &SliderWidgetBuilder) -> Self {
        let slider = SliderWidget::create_slider(&builder);

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        container.add_css_class(&builder.container_class);
        container.add_css_class(&builder.main_class);

        let revealer = gtk4::Revealer::builder()
            .transition_type(gtk4::RevealerTransitionType::SlideUp)
            .transition_duration(builder.transition_duration)
            .build();

        revealer.set_child(Some(&slider));
        revealer.add_css_class(&builder.main_class);

        let label = gtk4::Label::new(Some(&builder.icons[0]));

        label.add_css_class(&builder.label_class);
        label.add_css_class(&builder.main_class);
        
        container.append(&revealer);
        container.append(&label);

        let motion_controller = gtk4::EventControllerMotion::new();
        container.add_controller(motion_controller.clone());

        let max_value = builder.max_value;
        let callback  = builder.set_value; 

        let value_changed_signal = slider.connect_value_changed(move |scale| {
            (callback)(scale.value() / max_value);
        });

        let revealer_clone_1 = revealer.clone();
        let revealer_clone_2 = revealer.clone();

        motion_controller.connect_enter(move |_, _, _| {
            revealer_clone_1.set_reveal_child(true);
        });

        motion_controller.connect_leave(move |_| {
            revealer_clone_2.set_reveal_child(false);
        });

        let widget = SliderWidget {
            slider: slider.clone(),
            label: label.clone(),
            container,
            value_changed_signal: Rc::new(value_changed_signal),
            max_value: builder.max_value,
            icons:     builder.icons.clone(),
            get_value: builder.get_value,
            click:     builder.click,
        };

        let widget_clone_1 = widget.clone();
        let widget_clone_2 = widget.clone();

        let gesture = gtk4::GestureClick::new();
        gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            (widget_clone_1.click)();
            widget_clone_1.update_slider((widget_clone_1.get_value)());
        });

        label.add_controller(gesture);

        slider.connect_value_changed(move |scale| {
            widget_clone_2.update_button(scale.value() / widget_clone_2.max_value);
        });

        widget
    }

    fn create_slider(builder: &SliderWidgetBuilder) -> gtk4::Scale {
        let scale_adjustment = gtk4::Adjustment::new(
            0.0,               // Initial value
            0.0,               // Lower bound
            builder.max_value, // Upper bound
            5.0,               // Step increment
            0.0,               // Page increment
            0.0,               // Page size
        );

        let slider = gtk4::Scale::builder()
            .adjustment(&scale_adjustment)
            .orientation(gtk4::Orientation::Vertical)
            .height_request(builder.slider_height)
            .build();

        slider.add_css_class(&builder.slider_class);
        slider.add_css_class(&builder.main_class);
        slider.set_inverted(true);

        slider
    }

    fn update_slider(&self, value: f64) {
        self.slider.block_signal(&self.value_changed_signal);
        self.slider.set_value(value * 100.0);
        self.slider.unblock_signal(&self.value_changed_signal);
    }

    fn update_button(&self, value: f64) {
        if value < EPS || self.icons.len() == 1 {
            self.label.set_text(&self.icons[0]);
        } else {
            self.label.set_text(&self.icons[(value * (self.icons.len() - 1) as f64).ceil() as usize]);
        }
    }

}

impl Default for SliderWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl BarWidget for SliderWidget {

    fn update_widget(&mut self) {
        let sound_value = (self.get_value)();

        self.update_slider(sound_value);
    }

    fn bind_widget(&self, container: &impl BoxExt) {
        container.append(&self.container);
    }
}

