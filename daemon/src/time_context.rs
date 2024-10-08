use std::{io::ErrorKind, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeDelta};
use tokio::sync::Mutex;

use crate::server_context::{EventHandler, RsbarContext, RsbarContextContent};

const TIME_PRECISION_SEC: i64 = 2;
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
        let now = Local::now();

        if now.time() - self.time.time() >= TimeDelta::seconds(TIME_PRECISION_SEC){
            self.time = now;

            self.force_events().await?;
        }

        Ok(())
    }

    async fn call(&mut self, procedure: &str, _args: &str) -> Option<String> {
        let _ = self.update().await;

        match procedure {
            "time"   => Some(self.time.format(TIME_FORMAT).to_string()),
            _ => None,
        }
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
