use spacetimedb::reducer;
use spacetimedb::ReducerContext;
use spacetimedb::Table;

use crate::config::config;
use crate::config::Config;
use crate::config::ConfigId;
use crate::taskbar_remote_kind::TaskbarRemoteKind;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Called when the module is initially published
    // Set the default configuration
    ctx.db.config().insert(Config {
        id: ConfigId::default(),
        taskbar_remote: TaskbarRemoteKind::Windows,
    });
}
