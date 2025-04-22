use spacetimedb::ReducerContext;

use crate::config::config;
use crate::config::Config;
use crate::config::ConfigId;

pub fn get_config(ctx: &ReducerContext) -> Result<Config, String> {
    match ctx.db.config().id().find(ConfigId::default()) {
        Some(config) => Ok(config.clone()),
        None => Err("No default configuration found. This is unexpected, the init reducer should have created this.".to_string()),
    }
}
