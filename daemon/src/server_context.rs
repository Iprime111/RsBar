use std::{collections::HashMap, io::ErrorKind, sync::Arc};

use tokio::sync::{mpsc, Mutex};

use crate::rsbar_context::{EventHandler, RsbarContext};

const EVENT_REQUEST_PARTS: usize = 2;
const CALL_REQUEST_PARTS:  usize = 3;

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

    pub async fn new_call(&mut self, request: &str) -> tokio::io::Result<()> {
        let request_parts = split_request(request, CALL_REQUEST_PARTS)?;

        if let Some(context) = self.contexts.get_mut(request_parts[0]) {
            (*context).context.call(request_parts[1], request_parts[2]).await?;
             
            return Ok(());
        }

        Err(std::io::Error::new(ErrorKind::Other, format!("Can't get context by name {}", request_parts[0])))
    }

    pub async fn new_event_client(&mut self, request: &str, stream: mpsc::Sender<String>) -> tokio::io::Result<()> {
        let request_parts = split_request(request, EVENT_REQUEST_PARTS)?;
        
        if let Some(context) = self.contexts.get_mut(request_parts[0]) {
            let _ = context.context.force_events().await;
            
            self.event_handler.lock().await.add_event(request, stream);
            
            return Ok(());
        }

        Err(std::io::Error::new(ErrorKind::Other, format!("Can't get context by name {}", request_parts[0])))
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

fn split_request(request: &str, right_parts_count: usize) -> tokio::io::Result<Vec<&str>> {
    let request_trimmed = request.trim();
    
    let request_parts: Vec<&str> = request_trimmed.split('/').collect();
    let parts_count = request_parts.len();

    if parts_count != right_parts_count {
        return Err(std::io::Error::new(ErrorKind::Other, format!("Invalid request parts count: {request_trimmed}")));
    }

    Ok(request_parts)
}
