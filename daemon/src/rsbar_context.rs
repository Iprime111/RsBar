use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use tokio::sync::{mpsc, Mutex};

pub struct EventHandler {
    events: HashMap<String, Vec<mpsc::Sender<String>>>,
} 

impl EventHandler {
    pub fn new() -> Self{
        return EventHandler {
            events: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, name: &str, client: mpsc::Sender<String>) {
        self.events.entry(name.to_string()).or_insert(Vec::new()).push(client);
    }

    pub async fn trigger_event(&self, name: &str, data: &str) {
        let clients = self.events.get(name);
        
        if clients.is_none() {
            return;
        }

        for client in clients.unwrap() {

            let response = format!("{}/{}", name, data);
            let _ = client.send(response).await;
        }
    }
}

#[async_trait]
pub trait RsbarContextContent {
    async fn init(&mut self, event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()>;
    async fn update(&mut self) -> tokio::io::Result<()>;

    async fn force_events(&mut self) -> tokio::io::Result<()>;

    // Event socket:
    // Event subscription format: "<context name>/<parameter name>"
    // Event response format:     "<parameter content>" or None in case of error

    // Method socket:
    // Call args format:   "<context name>/<procedure name>/<arg string>"
    async fn call(&mut self, procedure: &str, args: &str) -> tokio::io::Result<()>;
}

pub struct RsbarContext {
    pub context: Box<dyn RsbarContextContent + Send + Sync>,
}

impl RsbarContext {
    pub fn new(context: Box<dyn RsbarContextContent + Send + Sync>) -> Self {
        RsbarContext {
            context,
        }
    }
}
