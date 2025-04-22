use crate::module_bindings::Taskbar;

pub struct WindowsTaskbar {
    pub id: u32,
    pub rect: windows::Win32::Foundation::RECT,
    pub is_secondary: bool,
    pub apps: Vec<String>
}

impl From<WindowsTaskbar> for Taskbar {
    fn from(value: WindowsTaskbar) -> Self {
        Taskbar {
            id: value.id,
            is_secondary: value.is_secondary,
            x: value.rect.left,
            y: value.rect.top,
            width: (value.rect.right - value.rect.left) as u32,
            height: (value.rect.bottom - value.rect.top) as u32,
            apps: value.apps,
        }
    }
}
