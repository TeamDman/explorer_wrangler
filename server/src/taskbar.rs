use spacetimedb::table;

pub type TaskbarId = u32;

#[table(name = taskbar, public)]
pub struct Taskbar {
    #[primary_key]
    pub id: TaskbarId,
    pub is_secondary: bool,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub apps: Vec<String>,
}
