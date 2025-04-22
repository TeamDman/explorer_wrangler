use crate::windows_taskbar::WindowsTaskbar;
use eyre::Result;
use log::info;
use windows::core::*;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::FindWindowExW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

pub fn get_taskbars() -> Result<Vec<WindowsTaskbar>> {
    let mut rtn = Vec::new();

    let primary_class = to_string("Shell_TrayWnd");
    let secondary_class = to_string("Shell_SecondaryTrayWnd");

    println!("Enumerating taskbars:");

    // Find the primary taskbar
    let mut hwnd: HWND =
        unsafe { FindWindowExW(HWND(0), HWND(0), PCWSTR(primary_class.as_ptr()), None) };
    let mut i = 0;
    let mut id = 0;

    

    while hwnd.0 != 0 {
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(hwnd, &mut rect)?;
        }
        rtn.push(WindowsTaskbar { id, rect, is_secondary:false, apps: vec![], });
        id += 1;

        info!(
            "Taskbar {} (Primary): left={}, top={}, right={}, bottom={}",
            i, rect.left, rect.top, rect.right, rect.bottom
        );

        // Continue looking for more? (Should only be one primary)
        hwnd = unsafe { FindWindowExW(HWND(0), hwnd, PCWSTR(primary_class.as_ptr()), None) };
        i += 1;
    }

    // Now the secondary taskbars
    let mut hwnd =
        unsafe { FindWindowExW(HWND(0), HWND(0), PCWSTR(secondary_class.as_ptr()), None) };

    while hwnd.0 != 0 {
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(hwnd, &mut rect)?;
        }
        rtn.push(WindowsTaskbar { id, rect, is_secondary:true, apps: vec![], });
        info!(
            "Taskbar {} (Secondary): left={}, top={}, right={}, bottom={}",
            i, rect.left, rect.top, rect.right, rect.bottom
        );

        hwnd = unsafe { FindWindowExW(HWND(0), hwnd, PCWSTR(secondary_class.as_ptr()), None) };
        i += 1;
    }
    Ok(rtn)
}

fn to_string(name: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(name).encode_wide().chain(Some(0)).collect()
}
