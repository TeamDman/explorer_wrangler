use once_cell::sync::Lazy;
use windows::Win32::UI::Accessibility::UnhookWinEvent;
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::Accessibility::SetWinEventHook;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_NAMECHANGE;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS;
use windows::core::BOOL;

struct WindowInfo {
    rect: RECT,
    title: String,
}

// Global cache: HWND → (RECT, title)
static WINDOWS: Lazy<Mutex<HashMap<isize, WindowInfo>>> = Lazy::new(|| Mutex::new(HashMap::new())); // is this sound? HWND is not send...

/// Initial enumeration of all top‐level visible windows
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    unsafe {
        if IsWindowVisible(hwnd).ok().is_err() {
            return BOOL(1);
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let len = GetWindowTextLengthW(hwnd);
            let mut title = String::new();
            if len > 0 {
                let mut buf = vec![0u16; (len + 1) as usize];
                let copied = GetWindowTextW(hwnd, &mut buf);
                if copied > 0 {
                    buf.truncate(copied as usize);
                    title = String::from_utf16_lossy(&buf);
                }
            }
            WINDOWS
                .lock()
                .unwrap()
                .insert(hwnd.0 as isize, WindowInfo { rect, title });
        }
        BOOL(1)
    }
}

/// WinEvent callback: only location‐ or name‐change on top‐level windows
unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _thread_id: u32,
    _time: u32,
) {
    // filter out non‐window or child events
    if hwnd.is_invalid() || id_object != 0 || id_child != 0 {
        return;
    }
    // only handle move/size or title‐change
    if event != EVENT_OBJECT_LOCATIONCHANGE && event != EVENT_OBJECT_NAMECHANGE {
        return;
    }
    // if it went invisible, remove it
    if unsafe { IsWindowVisible(hwnd) }.ok().is_err() {
        WINDOWS.lock().unwrap().remove(&(hwnd.0 as isize));
        return;
    }
    // otherwise re‐query its rect & title
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.ok().is_none() {
        return;
    }
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    let mut title = String::new();
    if len > 0 {
        let mut buf = vec![0u16; (len + 1) as usize];
        let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
        if copied > 0 {
            buf.truncate(copied as usize);
            title = String::from_utf16_lossy(&buf);
        }
    }
    WINDOWS
        .lock()
        .unwrap()
        .insert(hwnd.0 as isize, WindowInfo { rect, title });
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // 1) seed our cache
    unsafe {
        EnumWindows(Some(enum_windows_proc), LPARAM(0))?;
    }

    // 2) install a global hook for move/size & title‐change
    let hook: HWINEVENTHOOK = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_NAMECHANGE,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };

    // 3) spawn a thread that prints our cache every 500 ms
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_millis(500));
            let map = WINDOWS.lock().unwrap();
            println!("-- Window List --");
            for (hwnd, info) in map.iter() {
                println!(
                    "HWND: 0x{:X}, Pos: ({}, {}, {}, {}), Title: \"{}\"",
                    hwnd,
                    info.rect.left,
                    info.rect.top,
                    info.rect.right,
                    info.rect.bottom,
                    info.title
                );
            }
        }
    });

    // 4) keep the main thread alive
    thread::park();

    // never reached in this example, but good hygiene:
    unsafe { UnhookWinEvent(hook).unwrap(); }
    Ok(())
}
