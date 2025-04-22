use spacetimedb::table;
use spacetimedb::Identity;

use crate::taskbar_remote_kind::TaskbarRemoteKind;

pub type TaskbarId = Identity;

#[table(name = taskbar, public)]
pub struct Taskbar {
    #[primary_key]
    pub id: TaskbarId,
    pub remote: TaskbarRemoteKind,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}
