mod get_taskbars;
mod module_bindings;
pub mod windows_taskbar;
use get_taskbars::get_taskbars;
// Where your generated code will be
use module_bindings::*;
use simple_logger::SimpleLogger;
use spacetimedb_sdk::DbContext;
use spacetimedb_sdk::Identity;

// Define Constants
const DB_NAME: &str = "explorer-wrangler";
const DB_ADDRESS: &str = "http://localhost:3000";

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    // Initialize a basic logger.
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("Failed to initialize logger");

    // Build connection
    let connection = DbConnection::builder()
        .on_connect(on_connected)
        .on_connect_error(on_connect_error)
        .on_disconnect(on_disconnected)
        .with_uri(DB_ADDRESS)
        .with_module_name(DB_NAME)
        .build()
        .expect("Failed to build connection");

    // Subscribe to taskbars to view changes
    subscribe_to_tables(&connection);

    //Run connection in thread
    connection.run_threaded();

    // Main program loop or other logic here
    loop {
        sync_taskbars(&connection)?;
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

//Connection status changes
fn on_connected(_conn: &DbConnection, _who: Identity, _token: &str) {
    log::info!("Client connected to SpacetimeDB!");
}

fn on_connect_error(_err_ctx: &ErrorContext, err: spacetimedb_sdk::Error) {
    log::error!("Failed to connect: {}", err);
}

fn on_disconnected(_err_ctx: &ErrorContext, err: Option<spacetimedb_sdk::Error>) {
    log::info!(
        "Client disconnected: {}",
        err.map_or("Clean disconnect".into(), |e| e.to_string())
    );
}

// Subscribing to taskbars to view
fn subscribe_to_tables(connection: &DbConnection) {
    let _sub = connection
        .subscription_builder()
        .on_applied(on_subscribed)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM taskbar"]);
    log::info!("Subscribed to Taskbars!");
}

fn on_subscribed(_ctx: &SubscriptionEventContext) {
    log::info!("Successfully Subscribed!");
}

fn on_sub_error(_err_ctx: &ErrorContext, err: spacetimedb_sdk::Error) {
    log::error!("Subscription error: {:?}", err);
}

// Calling reducer functions
fn sync_taskbars(connection: &DbConnection) -> eyre::Result<()> {
    let taskbars = get_taskbars()?;
    connection
        .reducers
        .sync_taskbars(taskbars.into_iter().map(|x| x.into()).collect())?;
    log::info!("Synced taskbars!");
    Ok(())
}
