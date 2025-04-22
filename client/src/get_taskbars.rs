use crate::windows_taskbar::WindowsTaskbar;
use eyre::Result;
use log::info;
use windows::core::*;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
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

    while hwnd.0 != 0 {
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(hwnd, &mut rect)?;
        }
        let apps = get_taskbar_apps(hwnd);

        let taskbar = WindowsTaskbar {
            id: i,
            rect,
            is_secondary: false,
            apps,
        };
        i += 1;
        info!("Taskbar {:?}", taskbar);

        rtn.push(taskbar);

        info!(
            "Taskbar {} (Primary): left={}, top={}, right={}, bottom={}",
            i, rect.left, rect.top, rect.right, rect.bottom
        );

        // Continue looking for more? (Should only be one primary)
        hwnd = unsafe { FindWindowExW(HWND(0), hwnd, PCWSTR(primary_class.as_ptr()), None) };
    }

    // Now the secondary taskbars
    let mut hwnd =
        unsafe { FindWindowExW(HWND(0), HWND(0), PCWSTR(secondary_class.as_ptr()), None) };

    while hwnd.0 != 0 {
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(hwnd, &mut rect)?;
        }

        let apps = get_taskbar_apps(hwnd);
        let taskbar = WindowsTaskbar {
            id: i,
            rect,
            is_secondary: true,
            apps,
        };
        info!("Taskbar {:?}", taskbar);
        i += 1;
        rtn.push(taskbar);

        hwnd = unsafe { FindWindowExW(HWND(0), hwnd, PCWSTR(secondary_class.as_ptr()), None) };
    }

    Ok(rtn)
}

fn to_string(name: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(name).encode_wide().chain(Some(0)).collect()
}

use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;

unsafe extern "system" fn enum_buttons_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let apps: &mut Vec<String> = &mut *(lparam.0 as *mut Vec<String>);

    let len = GetWindowTextLengthW(hwnd);
    if len > 0 {
        let mut buf = vec![0u16; len as usize + 1];
        let text_len = GetWindowTextW(hwnd, &mut buf);
        if text_len > 0 {
            if let Ok(s) = String::from_utf16(&buf[..text_len as usize]) {
                apps.push(s);
            }
        }
    }

    true.into()
}

fn get_taskbar_apps(taskbar_hwnd: HWND) -> Vec<String> {
    let mut apps = Vec::new();
    unsafe {
        EnumChildWindows(
            taskbar_hwnd,
            Some(enum_buttons_proc),
            LPARAM(&mut apps as *mut _ as isize),
        );
    }
    apps
}
