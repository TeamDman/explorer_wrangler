use spacetimedb::reducer;
use spacetimedb::Identity;
use spacetimedb::ReducerContext;
use spacetimedb::Table;

use crate::system_tray::SystemTrayId;
use crate::system_tray_entry::system_tray_entry;
use crate::system_tray_entry::SystemTrayEntry;
use crate::system_tray_entry::SystemTrayEntryId;

#[reducer]
pub fn add_system_tray_entry(
    ctx: &ReducerContext,
    system_tray_entry_id: SystemTrayEntryId,
    system_tray_id: SystemTrayId,
    icon: String,
    tooltip: String,
    text: String,
) -> Result<(), String> {
    //Insert taskbar only if it doesn't already exist
    if ctx
        .db
        .system_tray_entry()
        .id()
        .find(&system_tray_id)
        .is_none()
    {
        ctx.db.system_tray_entry().insert(SystemTrayEntry {
            id: system_tray_id,
            system_tray_id,
            icon,
            tooltip,
            text,
        });
        Ok(())
    } else {
        Err("SystemTrayEntry with this ID already exists.".to_string())
    }
}
