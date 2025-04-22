use crate::config::Config;
use crate::get_config::get_config;
use crate::taskbar::taskbar;
use crate::taskbar::Taskbar;
use crate::taskbar::TaskbarId;
use spacetimedb::reducer;
use spacetimedb::ReducerContext;
use spacetimedb::Table;

#[reducer]
pub fn add_taskbar(
    ctx: &ReducerContext,
    taskbar_id: TaskbarId,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let config: Config = get_config(ctx)?;
    let remote = config.taskbar_remote;

    // Insert taskbar only if it doesn't already exist
    if ctx.db.taskbar().id().find(&taskbar_id).is_none() {
        ctx.db.taskbar().insert(Taskbar {
            id: taskbar_id,
            remote,
            width,
            height,
            x,
            y,
        });
        Ok(())
    } else {
        Err("Taskbar with this ID already exists.".to_string())
    }
}
