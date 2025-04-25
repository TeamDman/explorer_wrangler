use std::{thread, time::Duration};
use color_eyre::eyre::Result;
use tracing_subscriber;
use window_tracker::WindowTracker;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let tracker = WindowTracker::new()?;
    loop {
        thread::sleep(Duration::from_millis(500));
        println!("{}", tracker);
        // ... also pump your message loop here ...
    }
}
