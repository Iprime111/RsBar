mod time_widget;
mod bar_widget;
mod volume_widget;
mod slider_widget;
mod brightness_widget;
mod hyprland_workspaces_widget;
mod tokio_runtime;
mod unix_sockets;

use std::{fs, path::Path};

use bar_widget::BarWidget;
use brightness_widget::BrightnessWidget;
use log::error;
use tokio_runtime::tokio_runtime;
use unix_sockets::{setup_unix_sockets, ChannelsData};
use volume_widget::VolumeWidget;
use hyprland_workspaces_widget::HyprlandWorkspacesWidget;
use gtk4::{prelude::*, Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, LayerShell, Layer};
use time_widget::TimeWidget;

static CONFIG_PATH: &str = ".config/rsbar/style.css";

fn main() {
    colog::init();

    let channels_data = tokio_runtime().block_on(setup_unix_sockets()).unwrap();

    let app_id = format!("org.rsbar.bar");
    let app    = Application::builder().application_id(app_id).build();

    app.connect_startup(move |app| {
        let display = gtk4::gdk::Display::default().expect("Could not connect to a display.");

        let provider = gtk4::CssProvider::new();

        provider.load_from_string(&read_css_config());
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    
        tokio_runtime().block_on(build_ui(app, &channels_data));
    });
    
    app.run();
}

fn read_css_config() -> String {
    let home_folder_result = std::env::var("HOME"); 
    
    if home_folder_result.is_err() {
        error!("Unable to determine home folder path");
        panic!();
    }

    let full_config_path = format!("{}/{CONFIG_PATH}", home_folder_result.unwrap());
    let config_content_result = fs::read_to_string(Path::new(&full_config_path));

    if config_content_result.is_err() {
        error!("Unable to find config in {full_config_path}");
        panic!();
    }

    config_content_result.unwrap()
}

async fn build_ui(app: &Application, channels_data: &ChannelsData) {
    let display = gtk4::gdk::Display::default().expect("Could not connect to a display.");

    for monitor_id in 0..display.monitors().n_items() {
        build_window(app, &display, monitor_id, channels_data).await;
    }
}

async fn build_window(app: &Application, display: &gtk4::gdk::Display, monitor_id: u32, channels_data: &ChannelsData) {
    let monitor       = display.monitors().item(monitor_id).unwrap().downcast::<gtk4::gdk::Monitor>().unwrap();
    let screen_height = monitor.geometry().height();

    let top_box    = gtk4::Box::new(gtk4::Orientation::Vertical, 5);
    let middle_box = gtk4::Box::new(gtk4::Orientation::Vertical, 5);
    let bottom_box = gtk4::Box::new(gtk4::Orientation::Vertical, 5);

    top_box.set_valign(gtk4::Align::Start);
    middle_box.set_valign(gtk4::Align::Center);
    bottom_box.set_valign(gtk4::Align::End);

    top_box.set_vexpand(true);
    middle_box.set_vexpand(true);
    bottom_box.set_vexpand(true);


    let grid = gtk4::Grid::builder()
        .vexpand(true)
        .build();

    grid.attach(&top_box,    0, 0, 1, 1);
    grid.attach(&middle_box, 0, 1, 1, 1);
    grid.attach(&bottom_box, 0, 2, 1, 1);

    let time       = Box::new(TimeWidget::new());
    let volume     = Box::new(VolumeWidget::new(500));
    let brightness = Box::new(BrightnessWidget::new(500));
    let workspaces = Box::new(HyprlandWorkspacesWidget::new(9, 1));

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .default_width(30)// TODO
        .default_height(screen_height)
        .child(&grid)
        .build();
    
    window.add_css_class("main-window");

    setup_layer_shell(&window, &monitor);

    app.connect_activate(move |_| {
        window.present();
    });

    time.bind_widget(&top_box);
    workspaces.bind_widget(&middle_box);
    volume.bind_widget(&bottom_box);
    brightness.bind_widget(&bottom_box);
    
    let widgets: Vec<Box<dyn BarWidget>> = vec![time, workspaces, volume, brightness];

    for widget in widgets {
        let events = widget.events_list();

        for event in events {
            let _ = channels_data.event_subscription_tx.send(event.to_string()).await;
        }

        widget.bind_channels(channels_data.clone());
    }
}

fn setup_layer_shell(window: &ApplicationWindow, monitor: &gtk4::gdk::Monitor) {
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.auto_exclusive_zone_enable();

    let anchors = [
        (Edge::Left, true),
        (Edge::Right, false),
        (Edge::Top, false),
        (Edge::Bottom, false),
    ];

     for (anchor, state) in anchors {
        window.set_anchor(anchor, state);
    }

    window.set_monitor(monitor);
}
