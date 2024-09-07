use core::str;
use std::{env, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::UnixStream, sync::{mpsc, Mutex}};

use crate::server_context::{EventHandler, RsbarContext, RsbarContextContent};

//--------------------------------------------------------------------------------------------------------------------------------
//---------------------------------------------------------[ Globals ]------------------------------------------------------------
//--------------------------------------------------------------------------------------------------------------------------------

static XDG_RUNTIME_DIR:             Lazy<String> = Lazy::new(|| env::var("XDG_RUNTIME_DIR").unwrap());
static HYPRLAND_INSTANCE_SIGNATURE: Lazy<String> = Lazy::new(|| env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap());

static HYPRCTL_SOCKET: Lazy<String> = Lazy::new(|| 
    format!("{}/hypr/{}/.socket.sock", xdg_runtime_dir(), hyprland_instance_signature()));

static EVENT_SOCKET: Lazy<String> = Lazy::new(||
    format!("{}/hypr/{}/.socket2.sock", xdg_runtime_dir(), hyprland_instance_signature()));

//--------------------------------------------------------------------------------------------------------------------------------
//----------------------------------------------------------[ Context ]-----------------------------------------------------------
//--------------------------------------------------------------------------------------------------------------------------------

pub struct HyprlandContext {
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Workspace {
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

#[async_trait]
impl RsbarContextContent for HyprlandContext {
    async fn init(&mut self, event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()>{

        init_lazy_cells();

        tokio::spawn(Self::hyprland_event_listener_async(event_handler));

        Ok(())
    }

    async fn update(&mut self) -> tokio::io::Result<()> {
        Ok(())
    }

    fn call(&mut self, procedure: &str, args: &str) -> Option<String> {
        // TODO change workspace number
        match procedure {
            "setWorkspace" => { 
                Self::make_hyprctl_request(&format!("dispatch workspace {}", args)); 
            },
            _ => return None,
        };

        Some(args.to_string())
    }
}

impl HyprlandContext {
    pub fn new() -> (String, RsbarContext) {
        let new_context = Box::new(HyprlandContext {});

        ("hyprland".to_string(), RsbarContext::new(new_context))
    }

    async fn hyprland_event_listener_async(event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()> {
        let mut stream = UnixStream::connect(event_socket()).await?;
        let mut buffer = [0; 4096];
        
        let workspace = Self::get_active_workspace_async().await?;
        event_handler.lock().await.trigger_event("hyprland/workspace", &workspace.to_string()).await;
        
        loop {
            let bytes_count = stream.read(&mut buffer).await?;
            
            if bytes_count == 0 {
                break;
            }
        
            let response = String::from_utf8_lossy(&buffer[..bytes_count]);
            let event_strings = response.split('\n');
        
            for event in event_strings {
                if event.starts_with("workspace") || event.starts_with("focusedmon") {
        
                    // TODO function
                    let workspace = Self::get_active_workspace_async().await?;
                    event_handler.lock().await.trigger_event("hyprland/workspace", &workspace.to_string()).await;

                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn get_active_workspace_async() -> tokio::io::Result<i32> {
        
        let response = Self::make_hyprctl_request(&"j/activeworkspace".to_string()).await?;
    
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
}

//------------------------------------------------------------------------------------------------------------------------------
//---------------------------------------------------[ Lazy cells ]-------------------------------------------------------------
//------------------------------------------------------------------------------------------------------------------------------
//
fn init_lazy_cells() {
    Lazy::force(&XDG_RUNTIME_DIR);
    Lazy::force(&HYPRLAND_INSTANCE_SIGNATURE);

    Lazy::force(&HYPRCTL_SOCKET);
    Lazy::force(&EVENT_SOCKET);
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
