use chrono::DateTime;
use chrono::Local;
use once_cell::sync::Lazy;
use tracing::trace;
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tracing::debug;
use tracing::info;
use tracing::level_filters::LevelFilter;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::Accessibility::SetWinEventHook;
use windows::Win32::UI::Accessibility::UnhookWinEvent;
use windows::Win32::UI::WindowsAndMessaging::DispatchMessageW;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_NAMECHANGE;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::GetMessageW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows::Win32::UI::WindowsAndMessaging::MSG;
use windows::Win32::UI::WindowsAndMessaging::TranslateMessage;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS;
use windows::core::BOOL;

pub enum InvocationCacheStrategy {
    CacheFor5Seconds,
}

struct WindowInfo {
    rect: Option<RECT>,
    title: Option<String>,
    timestamp: DateTime<Local>,
}

pub enum EnumerationResult {
    ContinueEnumeration,
    StopEnumeration,
}

/// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms633498(v=vs.85)#return-value
impl From<EnumerationResult> for BOOL {
    fn from(result: EnumerationResult) -> Self {
        match result {
            EnumerationResult::ContinueEnumeration => BOOL(1),
            EnumerationResult::StopEnumeration => BOOL(0),
        }
    }
}

// Global cache: HWND → (RECT, title)
static WINDOWS: Lazy<Mutex<HashMap<isize, WindowInfo>>> = Lazy::new(|| Mutex::new(HashMap::new())); // is this sound? HWND is not send...

/// Initial enumeration of all top‐level visible windows
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    debug!("Gathering info for window with hwnd: {:?}", hwnd);
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return EnumerationResult::ContinueEnumeration.into();
        }
        let rect = get_window_rect(hwnd);
        let title = get_window_title(hwnd);
        WINDOWS.lock().unwrap().insert(
            hwnd.0 as isize,
            WindowInfo {
                rect,
                title,
                timestamp: Local::now(),
            },
        );
        return EnumerationResult::ContinueEnumeration.into();
    }
}

fn get_window_rect(hwnd: HWND) -> Option<RECT> {
    let rect = {
        let mut rect = RECT::default();
        if !unsafe { GetWindowRect(hwnd, &mut rect) }.is_ok() {
            None
        } else {
            Some(rect)
        }
    };
    rect
}

fn get_window_title(hwnd: HWND) -> Option<String> {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    let mut title = None;
    if len > 0 {
        let mut buf = vec![0u16; (len + 1) as usize];
        let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
        if copied > 0 {
            buf.truncate(copied as usize);
            title = Some(String::from_utf16_lossy(&buf));
        }
    }
    title
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
    trace!(
        "WinEvent: hwnd: {:?}, event: {}, id_object: {}, id_child: {}",
        hwnd, event, id_object, id_child
    );
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
    let rect = get_window_rect(hwnd);
    let title = get_window_title(hwnd);
    info!(
        "Window changed: hwnd: {:?}, rect: {:?}, title: {:?}",
        hwnd, rect, title
    );
    WINDOWS.lock().unwrap().insert(
        hwnd.0 as isize,
        WindowInfo {
            rect,
            title,
            timestamp: Local::now(),
        },
    );
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_line_number(true)
        .with_file(true)
        .with_max_level(LevelFilter::DEBUG)
        .init();

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
                    "HWND: 0x{:X}, Pos: {}, Title: \"{:?}\"",
                    hwnd,
                    if let Some(rect) = info.rect {
                        format!(
                            "({}, {}, {}, {})",
                            rect.left, rect.top, rect.right, rect.bottom,
                        )
                    } else {
                        format!("(unknown)")
                    },
                    info.title
                );
            }
        }
    });


    // 4) enter the message loop
    let mut msg = MSG::default();
    loop {
        // Block until a message is received.
        if !unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            break;
        }
        info!("Message received: {:?}", msg);
        unsafe { TranslateMessage(&msg) }.ok()?;
        unsafe { DispatchMessageW(&msg) };
    }

    // never reached in this example, but good hygiene:
    unsafe {
        UnhookWinEvent(hook).unwrap();
    }
    Ok(())
}
