use tracing::level_filters::LevelFilter;
use window_tracker::WindowTracker;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_line_number(true)
        .with_file(true)
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let window_tracker = WindowTracker::new()?;
    let mut old = window_tracker.windows();
    loop {
        let new = window_tracker.windows();
        std::thread::sleep(std::time::Duration::from_millis(500));
        if new == old {
            continue;
        }
        println!("{}", window_tracker);
        old = new;
    }
}
