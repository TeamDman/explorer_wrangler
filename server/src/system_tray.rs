use spacetimedb::table;
use spacetimedb::Identity;
use spacetimedb::SpacetimeType;
use crate::taskbar::TaskbarId;

pub type SystemTrayId = Identity;

#[table(name = system_tray, public)]
pub struct SystemTray {
    #[primary_key]
    pub system_tray_id: SystemTrayId,
    pub taskbar_id: TaskbarId,
}
