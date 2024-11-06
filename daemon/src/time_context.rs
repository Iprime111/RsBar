use std::{io::ErrorKind, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Local};
use tokio::sync::Mutex;

use crate::rsbar_context::{EventHandler, RsbarContext, RsbarContextContent};

const TIME_FORMAT: &str = "%H\n%M";

pub struct TimeContext {
    time:          DateTime<Local>,
    event_handler: Option<Arc<Mutex<EventHandler>>>,
}

#[async_trait]
impl RsbarContextContent for TimeContext {
    async fn init(&mut self, event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()> {
        self.time = Local::now();
        self.event_handler = Some(event_handler);

        self.update().await?;

        Ok(())
    }

    async fn update(&mut self) -> tokio::io::Result<()> {
        self.time = Local::now();

        self.force_events().await?;

        Ok(())
    }

    async fn call(&mut self, _procedure: &str, _args: &str) -> tokio::io::Result<()> {
        Err(std::io::Error::new(ErrorKind::NotFound, "Time context does not support calls"))
    }

    async fn force_events(&mut self) -> tokio::io::Result<()> {
        if self.event_handler.is_none() {
            return Err(std::io::Error::new(ErrorKind::NotFound, "Event handler was not found"));
        }

        self.event_handler.as_mut().unwrap().lock().await
            .trigger_event("time/time", &self.time.format(TIME_FORMAT).to_string()).await;

        Ok(())
    }
}

impl TimeContext {
    pub fn new() -> (String, RsbarContext) {
        let new_context = Box::new(TimeContext {
            time: Local::now(),
            event_handler: None,
        });

        ("time".to_string(), RsbarContext::new(new_context))
    }
}
