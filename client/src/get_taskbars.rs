use crate::windows_taskbar::WindowsTaskbar;
use eyre::bail;
use windows::core::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn get_taskbars() -> eyre::Result<Vec<WindowsTaskbar>> {
    unsafe {
        let hwnd = FindWindowW(w!("Shell_TrayWnd"), None);
        if hwnd.0 == 0 {
            bail!("Failed to find taskbar window");
        }

        let mut rect: windows::Win32::Foundation::RECT = Default::default();
        GetWindowRect(hwnd, &mut rect)?;

        Ok(vec![WindowsTaskbar { rect }])
    }
}
