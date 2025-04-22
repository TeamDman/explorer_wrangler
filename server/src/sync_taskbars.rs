use crate::taskbar::taskbar;
use crate::taskbar::Taskbar;
use log::info;
use spacetimedb::reducer;
use spacetimedb::ReducerContext;
use spacetimedb::Table;

#[reducer]
pub fn sync_taskbars(ctx: &ReducerContext, taskbars: Vec<Taskbar>) -> Result<(), String> {
    for taskbar in taskbars {
        let taskbar_id = taskbar.id.clone();
        if let Some(_taskbar) = ctx.db.taskbar().id().find(&taskbar_id) {
            ctx.db.taskbar().id().update(taskbar);
            info!("Taskbar with ID {} updated.", taskbar_id);
        } else {
            ctx.db.taskbar().insert(taskbar);
            info!("Taskbar with ID {} inserted.", taskbar_id);
        }
    }
    Ok(())
}
