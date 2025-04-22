use log::error;
use log::info;
use log::warn;
use spacetimedb::reducer;
use spacetimedb::ReducerContext;

use crate::config::config;
use crate::config::Config;
use crate::config::ConfigId;
use crate::taskbar_remote_kind::TaskbarRemoteKind;

#[reducer]
pub fn set_taskbar_remote(ctx: &ReducerContext, remote: TaskbarRemoteKind) -> Result<(), String> {
    let config_id = ConfigId::default();

    if let Some(config) = ctx.db.config().id().find(&config_id) {
        // Use `inner` for comparison, not the entire `ConfigId` struct
        if config.taskbar_remote != remote {
            info!("Changing taskbar remote to {:?}", remote);
            ctx.db.config().id().update(Config {
                id: config_id,
                taskbar_remote: remote,
            });
            Ok(())
        } else {
            warn!("Taskbar remote already set to {:?}", remote);
            Ok(())
        }
    } else {
        error!("No config found with id 0 to change remote.");
        Err("No default configuration found. This is unexpected, the init reducer should have created this.".to_string())
    }
}
