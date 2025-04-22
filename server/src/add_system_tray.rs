use crate::system_tray::system_tray;
use crate::system_tray::SystemTray;
use crate::system_tray::SystemTrayId;
use crate::taskbar::TaskbarId;
use spacetimedb::reducer;
use spacetimedb::ReducerContext;
use spacetimedb::Table;

#[reducer]
pub fn add_system_tray(
    ctx: &ReducerContext,
    system_tray_id: SystemTrayId,
    taskbar_id: TaskbarId,
) -> Result<(), String> {
    //Insert taskbar only if it doesn't already exist
    if ctx
        .db
        .system_tray()
        .system_tray_id()
        .find(SystemTrayId::default())
        .is_none()
    {
        ctx.db.system_tray().insert(SystemTray {
            system_tray_id,
            taskbar_id,
        });
        Ok(())
    } else {
        Err("SystemTray with this ID already exists.".to_string())
    }
}
