use std::{env, usize};

use gtk4::prelude::{GridExt, WidgetExt, GestureExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::UnixStream, sync::mpsc::{self, Sender}};

use crate::{bar_widget::BarWidget, tokio_runtime::tokio_runtime};

static XDG_RUNTIME_DIR:             Lazy<String> = Lazy::new(|| env::var("XDG_RUNTIME_DIR").unwrap());
static HYPRLAND_INSTANCE_SIGNATURE: Lazy<String> = Lazy::new(|| env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap());

static HYPRCTL_SOCKET: Lazy<String> = Lazy::new(|| 
    format!("{}/hypr/{}/.socket.sock", xdg_runtime_dir(), hyprland_instance_signature()));

static EVENT_SOCKET: Lazy<String> = Lazy::new(||
    format!("{}/hypr/{}/.socket2.sock", xdg_runtime_dir(), hyprland_instance_signature()));

pub struct HyprlandWorkspacesWidget {
    container:      gtk4::Grid,
    last_workspace: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id:                i32,
    pub name:              String,
    pub monitor:           String,

    #[serde(rename = "monitorID")]
    pub monitor_id:        i128,

    pub windows:           u16,
    #[serde(rename = "hasfullscreen")]
    pub fullscreen:        bool,
    
    #[serde(rename = "lastwindow")]
    pub last_window:       String,
    
    #[serde(rename = "lastwindowtitle")]
    pub last_window_title: String,
}

impl HyprlandWorkspacesWidget {
    pub fn new(rows: usize, cols: usize) -> Self {
        Lazy::force(&XDG_RUNTIME_DIR);
        Lazy::force(&HYPRLAND_INSTANCE_SIGNATURE);

        Lazy::force(&HYPRCTL_SOCKET);
        Lazy::force(&EVENT_SOCKET);

        let mut buttons: Vec<gtk4::Label> = Vec::new();
        let container = gtk4::Grid::builder()
            .row_homogeneous(true)
            .row_spacing(6)
            .column_spacing(2)
            .build();

        container.add_css_class("hyprland-workspaces-widget-container");
        container.add_css_class("hyprland-workspaces-widget");

        for row in 0..rows {
            for col in 0..cols {
                let button_number = row * cols + col;

                buttons.push(gtk4::Label::new(Some(format!("{}", button_number + 1).as_str())));

                buttons[button_number].add_css_class("hyprland-workspaces-widget-button");
                buttons[button_number].add_css_class("hyprland-workspaces-widget");

                let gesture = gtk4::GestureClick::new();
                gesture.connect_released(move |gesture, _, _, _| {
                    gesture.set_state(gtk4::EventSequenceState::Claimed);

                    tokio_runtime().spawn(async move {
                        let arg = format!("dispatch workspace {}", button_number + 1);

                        let _ = make_hyprctl_request(&arg).await;
                    });
                });

                buttons[button_number].add_controller(gesture);

                container.attach(&buttons[button_number], col as i32, row as i32,  1, 1);
            }
        }

        let (tx, mut rx) = mpsc::channel::<i32>(32);

        tokio_runtime().spawn(hyprland_event_listener_async(tx));

        let mut widget = Self {
            container,
            last_workspace: 1,
        };

        gtk4::glib::spawn_future_local(async move {
            while let Some(workspace_id) = rx.recv().await {
                if workspace_id <= 0 || workspace_id as usize > buttons.len() {
                    continue;
                }

                buttons[(workspace_id - 1) as usize].add_css_class("hyprland-workspaces-widget-picked");
                buttons[widget.last_workspace - 1].remove_css_class("hyprland-workspaces-widget-picked");

                widget.last_workspace = workspace_id as usize;
            }
        });

        widget
    }
}

impl BarWidget for HyprlandWorkspacesWidget {
    fn update_widget(&mut self) {}

    fn bind_widget(&self, container: &impl gtk4::prelude::BoxExt) {
        container.append(&self.container);
    }
}

async fn hyprland_event_listener_async(tx: Sender<i32>) -> tokio::io::Result<()> {
    let mut stream = UnixStream::connect(event_socket()).await?;
    let mut buffer = [0; 4096];

    let workspace = get_active_workspace_async().await?;
    tx.send(workspace).await.unwrap();

    
    loop {
        let bytes_count = stream.read(&mut buffer).await?;
        if bytes_count == 0 {
            break;
        }
    
        let response = String::from_utf8_lossy(&buffer[..bytes_count]);
        let event_strings = response.split('\n');
    
        for event in event_strings {
            if event.starts_with("workspace") || event.starts_with("focusedmon") {

                let workspace = get_active_workspace_async().await?;
                tx.send(workspace).await.unwrap();
    
                break;
            }
        }
    
    }

    Ok(())
}

async fn get_active_workspace_async() -> tokio::io::Result<i32> {
    
    let response = make_hyprctl_request(&"j/activeworkspace".to_string()).await?;

    let deserialized: Workspace = serde_json::from_str(&response)?;
    
    Ok(deserialized.id)
}

async fn make_hyprctl_request(request: &String) -> tokio::io::Result<String> {
    let mut stream = UnixStream::connect(hyprctl_socket()).await?;

    let _ = stream.write_all(request.as_bytes()).await?;

    let mut buf = [0; 8192]; //NOTE buffer size is taken from hyprctl sources
    let bytes_count = stream.read(&mut buf).await?;
    
    let response = String::from_utf8(buf[..bytes_count].to_vec()).unwrap();

    Ok(response)
}

macro_rules! get_lazy {
    ($name:ident $var:expr) => {
        fn $name() -> &'static String {
            Lazy::get($var).unwrap()
        }
    };
}

get_lazy!(xdg_runtime_dir             &XDG_RUNTIME_DIR);
get_lazy!(hyprland_instance_signature &HYPRLAND_INSTANCE_SIGNATURE);
get_lazy!(hyprctl_socket              &HYPRCTL_SOCKET);
get_lazy!(event_socket                &EVENT_SOCKET);

