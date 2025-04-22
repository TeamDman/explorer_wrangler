use crate::config::Config;
use crate::get_config::get_config;
use crate::taskbar::taskbar;
use crate::taskbar::Taskbar;
use crate::taskbar::TaskbarId;
use spacetimedb::reducer;
use spacetimedb::ReducerContext;

#[reducer]
pub fn update_taskbar(
    ctx: &ReducerContext,
    id: TaskbarId,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let config: Config = get_config(ctx)?;
    let remote = config.taskbar_remote;

    //Update taskbar only if it exists
    if let Some(_taskbar) = ctx.db.taskbar().id().find(&id) {
        ctx.db.taskbar().id().update(Taskbar {
            id,
            remote,
            width,
            height,
            x,
            y,
        });
        Ok(())
    } else {
        Err("Taskbar with this ID doesn't exists.".to_string())
    }
}
