use chrono::DateTime;
use chrono::Local;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use tracing::debug;
use tracing::info;
use tracing::trace;
use windows::core::BOOL;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Accessibility::SetWinEventHook;
use windows::Win32::UI::Accessibility::UnhookWinEvent;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_NAMECHANGE;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS;

/// Holds the last‐seen geometry, title, and timestamp of a window.
#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub rect: Option<RECT>,
    pub title: Option<String>,
    pub timestamp: DateTime<Local>,
}

/// Our global in‐memory cache: HWND -> WindowInfo
static WINDOWS: Lazy<Mutex<HashMap<isize, WindowInfo>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Tracks all top‐level windows, updates on move/resize/title‐change,
/// and unhooks itself on drop.
pub struct WindowTracker {
    hook: HWINEVENTHOOK,
}

impl WindowTracker {
    /// Creates a new tracker: seeds the cache and installs the WinEvent hook.
    pub fn new() -> eyre::Result<Self> {
        // 1) initial enumeration
        unsafe {
            EnumWindows(Some(enum_windows_proc), LPARAM(0))?;
        }

        // 2) install global WinEvent hook
        let hook = unsafe {
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

        Ok(WindowTracker { hook })
    }

    /// Returns a snapshot of (hwnd, WindowInfo) pairs.
    pub fn windows(&self) -> Vec<(isize, WindowInfo)> {
        let guard = WINDOWS.lock().unwrap();
        guard
            .iter()
            .map(|(hwnd, info)| (*hwnd, info.clone()))
            .collect()
    }
}

impl fmt::Display for WindowTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "-- Window List --")?;
        for (hwnd, info) in self.windows() {
            let pos = info
                .rect
                .map(|r| format!("({}, {}, {}, {})", r.left, r.top, r.right, r.bottom))
                .unwrap_or_else(|| "unknown".into());
            writeln!(
                f,
                "HWND: 0x{:X}, Pos: {}, Title: {:?}",
                hwnd, pos, info.title
            )?;
        }
        Ok(())
    }
}

impl Drop for WindowTracker {
    fn drop(&mut self) {
        // unhook on drop
        unsafe {
            UnhookWinEvent(self.hook).ok();
        }
    }
}

/// Callback for the initial `EnumWindows`, just seeds our cache.
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    debug!("enum_windows_proc: {:?}", hwnd);
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    // get rect
    let rect = {
        let mut r = RECT::default();
        if GetWindowRect(hwnd, &mut r).is_ok() {
            Some(r)
        } else {
            None
        }
    };

    // get title
    let title = {
        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            let copied = GetWindowTextW(hwnd, &mut buf);
            if copied > 0 {
                buf.truncate(copied as usize);
                Some(String::from_utf16_lossy(&buf))
            } else {
                None
            }
        } else {
            None
        }
    };

    WINDOWS.lock().unwrap().insert(
        hwnd.0 as isize,
        WindowInfo {
            rect,
            title,
            timestamp: Local::now(),
        },
    );
    BOOL(1) // continue enumeration
}

/// WinEvent hook: updates or removes entries on move/resize/title‐change.
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
        "win_event_proc: hwnd={:?}, event={}, obj={}, child={}",
        hwnd,
        event,
        id_object,
        id_child
    );

    // only top‐level window changes
    if hwnd.is_invalid() || id_object != 0 || id_child != 0 {
        return;
    }
    if event != EVENT_OBJECT_LOCATIONCHANGE && event != EVENT_OBJECT_NAMECHANGE {
        return;
    }

    // if now invisible, drop it
    if !IsWindowVisible(hwnd).as_bool() {
        WINDOWS.lock().unwrap().remove(&(hwnd.0 as isize));
        return;
    }

    // otherwise re‐query and update
    let rect = {
        let mut r = RECT::default();
        if GetWindowRect(hwnd, &mut r).is_ok() {
            Some(r)
        } else {
            None
        }
    };
    let title = {
        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            let copied = GetWindowTextW(hwnd, &mut buf);
            if copied > 0 {
                buf.truncate(copied as usize);
                Some(String::from_utf16_lossy(&buf))
            } else {
                None
            }
        } else {
            None
        }
    };

    info!(
        "window changed: {:?} → rect={:?}, title={:?}",
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
