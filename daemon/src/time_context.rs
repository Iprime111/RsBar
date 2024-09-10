use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeDelta};
use tokio::sync::Mutex;

use crate::server_context::{EventHandler, RsbarContextContent};

const TIME_PRECISION_SEC: i64 = 60;

pub struct TimeContext {
    time:          DateTime<Local>,
    event_handler: Arc<Mutex<EventHandler>>,
}

#[async_trait]
impl RsbarContextContent for TimeContext {
    async fn init(&mut self, event_handler: Arc<Mutex<EventHandler>>) -> tokio::io::Result<()> {
        self.time = Local::now();
        self.event_handler = event_handler;

        self.update();

        Ok(())
    }

    async fn update(&mut self) -> tokio::io::Result<()> {
        let now = Local::now();

        if now.time() - self.time.time() >= TimeDelta::seconds(TIME_PRECISION_SEC){
            self.time = now;

            self.event_handler.lock().await.trigger_event("time/time", &self.time.format("%H\n%M").to_string());
        }

        Ok(())
    }

    async fn call(&mut self, procedure: &str, _args: &str) -> Option<String> {
        self.update();

        match procedure {
            "time"   => Some(self.time.format("%H\n%M").to_string()),
            _ => None,
        }
    }
}
