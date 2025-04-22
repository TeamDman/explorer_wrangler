use crate::system_tray::SystemTrayId;
use spacetimedb::table;
use spacetimedb::Identity;

pub type SystemTrayEntryId = Identity;

#[table(name = system_tray_entry, public)]
pub struct SystemTrayEntry {
    #[primary_key]
    pub id: SystemTrayEntryId,
    pub system_tray_id: SystemTrayId,
    pub icon: String,
    pub tooltip: String,
    pub text: String,
}
