use std::cell::RefCell;
use std::rc::Rc;

use gtk4::{glib::object::ObjectExt, prelude::GestureExt};
use gtk4::glib::{MainContext, SignalHandlerId};
use log::error;
use crate::bar_widget::BarWidget;
use crate::unix_sockets::ChannelsData;
use gtk4::prelude::{BoxExt, WidgetExt, RangeExt};

const EPS: f64 = 1e-5;

type GetterFunction = fn(&str, &str) -> SliderFetchResult;
type SetterFunction = fn(f64) -> String;
type ClickFunction  = fn() -> String;

pub enum SliderFetchResult {
    On,
    Off,
    Value(f64),
    None,
}

#[derive(Clone)]
pub struct SliderWidget {
    slider:               gtk4::Scale,
    label:                gtk4::Label,
    container:            gtk4::Box,
    max_value:            f64,
    icons:                Vec<String>,
    get_value:            GetterFunction,
    set_value:            SetterFunction,
    click:                ClickFunction,
}

pub struct SliderWidgetBuilder {
    max_value:           f64,
    icons:               Vec<String>,
    slider_height:       i32,
    transition_duration: u32,
    set_value:           SetterFunction,
    get_value:           GetterFunction,
    click:               ClickFunction,
    slider_class:        String,
    container_class:     String,
    label_class:         String,
    main_class:          String,
}

fn dummy_set(_: f64) -> String { String::new() }
fn dummy_get(_: &str, _: &str) -> SliderFetchResult { SliderFetchResult::Value(0.0) }
fn dummy_click() -> String { String::new() }

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
            max_value: builder.max_value,
            icons:     builder.icons.clone(),
            get_value: builder.get_value,
            set_value: builder.set_value,
            click:     builder.click,
        };

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
}
fn update_slider(slider: &gtk4::Scale, value_changed_signal: &SignalHandlerId, value: f64) {
    slider.block_signal(value_changed_signal);
    slider.set_value(value * 100.0);
    slider.unblock_signal(value_changed_signal);
}

fn update_button(label: &gtk4::Label, icons: &Vec<String>, value: f64) {
    if value < EPS || icons.len() == 1 {
        label.set_text(&icons[0]);
    } else {
        label.set_text(&icons[(value * (icons.len() - 1) as f64).ceil() as usize]);
    }
}

impl Default for SliderWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl BarWidget for SliderWidget {
    fn bind_widget(&self, container: &gtk4::Box) {
        container.append(&self.container);
    }

    fn events_list(&self) -> &'static[&'static str] {
        error!("events list must be specified manualy for each slider widget");
        panic!();
    }

    fn bind_channels(&self, mut channels_data: ChannelsData) {
        let widget_clone_1 = self.clone();
        let widget_clone_2 = self.clone();

        let call_tx_clone = channels_data.call_tx.clone();

        let gesture = gtk4::GestureClick::new();
        gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);

            let _ = channels_data.call_tx.send((widget_clone_1.click)());
        });
        self.label.add_controller(gesture);

        let is_on = Rc::new(RefCell::new(true));
        let is_on_clone = is_on.clone();

        let value_changed_signal = self.slider.connect_value_changed(move |scale| {
            let value = scale.value() / widget_clone_1.max_value;

            update_button(&widget_clone_1.label, &widget_clone_1.icons, value);
            let _ = call_tx_clone.send((widget_clone_1.set_value)(value));
            
            if value > EPS && !*is_on_clone.borrow() {
                let _ = call_tx_clone.send((widget_clone_1.click)());
                *is_on_clone.borrow_mut() = true;
            }
        });

        MainContext::default().spawn_local(async move {
            let mut value = 0.0;

            while let Ok(event) = channels_data.event_rx.recv().await {
                let new_value = (widget_clone_2.get_value)(&event.name, &event.value);
                
                match new_value {
                    SliderFetchResult::On =>  {
                        if !*is_on.borrow() {
                            *is_on.borrow_mut() = true;

                            update_slider(&widget_clone_2.slider, &value_changed_signal, value);
                            update_button(&widget_clone_2.label, &widget_clone_2.icons, value);
                        }
                    },
                    SliderFetchResult::Off => {
                        if *is_on.borrow() {
                            *is_on.borrow_mut() = false;

                            update_slider(&widget_clone_2.slider, &value_changed_signal, 0.0);
                            update_button(&widget_clone_2.label, &widget_clone_2.icons, 0.0);

                        }
                    },
                    SliderFetchResult::Value(slider_value) => {
                        value = slider_value;

                        if *is_on.borrow() {
                            update_slider(&widget_clone_2.slider, &value_changed_signal, slider_value);
                            update_button(&widget_clone_2.label, &widget_clone_2.icons, slider_value);
                        }
                    },
                    SliderFetchResult::None => continue,
                }
            }
        });
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

    pub fn set_value_callback(&mut self, callback: SetterFunction) -> &mut Self {
        self.set_value = callback;
        self
    }

    pub fn get_value_callback(&mut self, callback: GetterFunction) -> &mut Self {
        self.get_value = callback;
        self
    }

    pub fn click_callback(&mut self, callback: ClickFunction) -> &mut Self {
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

