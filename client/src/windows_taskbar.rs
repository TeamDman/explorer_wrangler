use crate::module_bindings::Taskbar;
use crate::module_bindings::TaskbarRemoteKind;

pub struct WindowsTaskbar {
    pub rect: windows::Win32::Foundation::RECT,
}

impl From<WindowsTaskbar> for Taskbar {
    fn from(value: WindowsTaskbar) -> Self {
        Taskbar {
            id: spacetimedb_sdk::Identity::default(),
            remote: TaskbarRemoteKind::Windows,
            x: value.rect.left,
            y: value.rect.top,
            width: (value.rect.right - value.rect.left) as u32,
            height: (value.rect.bottom - value.rect.top) as u32,
        }
    }
}
