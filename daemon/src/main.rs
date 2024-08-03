mod server_context;
mod volume_context;
mod brightness_context;

use server_context::ServerContext;

use tokio::time;
use volume_context::VolumeContext;
use std::time::Duration;

const POLLING_INTERVAL: u64 = 500; 

#[tokio::main]
async fn main() {
    let mut main_context = ServerContext::new();

    main_context.add_context(VolumeContext::new());

    let mut interval = time::interval(Duration::from_millis(POLLING_INTERVAL));

    // Update cycle
    loop {
        main_context.update();
        interval.tick().await;
    }
}
