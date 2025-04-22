use spacetimedb::table;
use spacetimedb::Identity;
use crate::taskbar_remote_kind::TaskbarRemoteKind;

pub type ConfigId = Identity;

#[table(name = config)]
#[derive(Clone)] // Added Clone derive
pub struct Config {
    #[primary_key]
    pub id: ConfigId,
    pub taskbar_remote: TaskbarRemoteKind,
}
