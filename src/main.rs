mod time_widget;
mod bar_widget;
mod volume_widget;
mod slider_widget;
mod brightness_widget;
mod hyprland_workspaces_widget;
mod tokio_runtime;

use bar_widget::BarWidget;
use brightness_widget::BrightnessWidget;
use volume_widget::VolumeWidget;
use hyprland_workspaces_widget::HyprlandWorkspacesWidget;
use gtk4::{prelude::*, Application, ApplicationWindow, glib};
use gtk4_layer_shell::{Edge, LayerShell, Layer};
use time_widget::TimeWidget;

const APP_ID: &str = "org.lBar.test";

fn main() {

    let app = Application::builder().application_id(APP_ID).build();

     app.connect_startup(|app| {
        let provider = gtk4::CssProvider::new();

        provider.load_from_string(include_str!("style.css"));
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        build_ui(app);
    });

    app.run();
}

fn build_ui(app: &Application) {
    let screen_height = 1080; //TODO

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

    let mut time       = TimeWidget::new();
    let mut volume     = VolumeWidget::new(500);
    let mut brightness = BrightnessWidget::new(500);
    let     workspaces = HyprlandWorkspacesWidget::new(9, 1);

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .default_width(30)// TODO
        .default_height(screen_height)
        .child(&grid)
        .build();

    window.add_css_class("main-window");

    setup_layer_shell(&window);

    app.connect_activate(move |_| {
        window.present();
    });

    time.bind_widget(&top_box);

    workspaces.bind_widget(&middle_box);
    
    volume.bind_widget(&bottom_box);
    brightness.bind_widget(&bottom_box);//TODO Vec
    

                                     
    let tick = move || {
        time.update_widget();
        volume.update_widget();
        brightness.update_widget();//TODO vec

        glib::ControlFlow::Continue
    };

    glib::timeout_add_seconds_local(1, tick);
}

fn setup_layer_shell(window: &ApplicationWindow) {
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
}
