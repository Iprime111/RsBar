use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex};

const EVENT_REQUEST_PARTS: usize = 2;
const CALL_REQUEST_PARTS:  usize = 3;

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
    // Call result format: "<return value>" or None in case of error
    async fn call(&mut self, procedure: &str, args: &str) -> Option<String>;
}

pub struct RsbarContext {
    context: Box<dyn RsbarContextContent + Send + Sync>,
}

impl RsbarContext {
    pub fn new(context: Box<dyn RsbarContextContent + Send + Sync>) -> Self {
        RsbarContext {
            context,
        }
    }
}

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

pub struct ServerContext {
    contexts:      HashMap<String, RsbarContext>,
    event_handler: Arc<Mutex<EventHandler>>,
}

impl ServerContext {
    pub fn new() -> Self {
        ServerContext { 
            contexts:      HashMap::new(),
            event_handler: Arc::new(Mutex::new(EventHandler::new())),
        }
    }

    pub async fn init(&mut self) -> tokio::io::Result<()>{
        for (_context_name, context) in self.contexts.iter_mut() {
            context.context.init(self.event_handler.clone()).await?;
        }

        Ok(())
    }

    pub async fn new_call(&mut self, request: &str) -> Option<String> {
        let request_parts = split_request(request, CALL_REQUEST_PARTS)?;

        let context = self.contexts.get_mut(request_parts[0])?;

        (*context).context.call(request_parts[1], request_parts[2]).await
    }

    // TODO error handling (err message)
    pub async fn new_event_client(&mut self, request: &str, stream: mpsc::Sender<String>) -> Option<()> {
        let request_parts = split_request(request, EVENT_REQUEST_PARTS)?;
        
        let context = self.contexts.get_mut(request_parts[0])?;

        let _ = context.context.force_events().await;

        self.event_handler.lock().await.add_event(request, stream);

        Some(())
    }

    pub fn add_context(&mut self, (context_name, context): (String, RsbarContext)) {
        self.contexts.insert(context_name, context);
    }

    pub async fn update(&mut self) -> tokio::io::Result<()> {
        for (_context_name, context) in &mut self.contexts {
            // TODO figure out how to run these updates concurrently           
            context.context.update().await?;
        };

        Ok(())
    }
}

fn split_request(request: &str, right_parts_count: usize) -> Option<Vec<&str>> {
    let request_trimmed = request.trim();
    
    let request_parts: Vec<&str> = request_trimmed.split('/').collect();
    let parts_count = request_parts.len();

    if parts_count != right_parts_count {
        return None;
    }

    Some(request_parts)
}
