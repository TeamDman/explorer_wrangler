use crate::type_conversion::AsBevyIRect;
use bevy_math::prelude::IRect;
use chrono::DateTime;
use chrono::Local;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::thread::{self};
use tracing::info;
use tracing::trace;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::RECT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::Threading::GetCurrentThreadId;
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
use windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW;
use windows::Win32::UI::WindowsAndMessaging::TranslateMessage;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
use windows::Win32::UI::WindowsAndMessaging::WINEVENT_SKIPOWNPROCESS;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
use windows::core::BOOL;

/// Holds the last‐seen geometry, title, and timestamp of a window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowInfo {
    pub rect: Option<IRect>,
    pub title: Option<String>,
    pub timestamp: DateTime<Local>,
}

/// Shared in‐memory cache: HWND → WindowInfo
static WINDOWS: Lazy<Mutex<HashMap<isize, WindowInfo>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Spawns a background thread that
///  1) Enumerates all top‐level windows,
///  2) Installs a WinEvent hook for moves/title‐changes,
///  3) Pumps a message loop,
///  4) Updates the `WINDOWS` cache,
/// and on Drop signals the thread to `WM_QUIT`, unhooks, and joins.
pub struct WindowTracker {
    thread_id: u32,
    handle: Option<JoinHandle<()>>,
}

impl WindowTracker {
    /// Starts the worker thread, blocks until initial enum & hook install finish.
    pub fn new() -> eyre::Result<Self> {
        let (tx, rx) = channel::<u32>();

        let handle = thread::spawn(move || {
            // 1) initial enumeration
            unsafe {
                EnumWindows(Some(enum_windows_proc), LPARAM(0)).unwrap();
            }

            // 2) install WinEvent hook
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

            // 3) signal ready (hook installed + initial cache seeded)
            let tid = unsafe { GetCurrentThreadId() };
            tx.send(tid).unwrap();

            // 4) pump messages until WM_QUIT
            let mut msg = MSG::default();
            while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
                unsafe { TranslateMessage(&msg) }.unwrap();
                unsafe { DispatchMessageW(&msg) };
            }

            // 5) cleanup hook
            unsafe { UnhookWinEvent(hook) }.ok().unwrap();
        });

        // wait for the thread to tell us it's ready
        let thread_id = rx.recv().unwrap();
        Ok(WindowTracker {
            thread_id,
            handle: Some(handle),
        })
    }

    /// Snapshot of current (HWND, WindowInfo).
    pub fn windows(&self) -> Vec<(isize, WindowInfo)> {
        WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(h, info)| (*h, info.clone()))
            .collect()
    }
}

impl fmt::Display for WindowTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "-- Window List --")?;
        for (hwnd, info) in self.windows() {
            let pos = info
                .rect
                .map(|r| format!("({r:?})"))
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
        // signal the worker thread to quit its message loop
        unsafe {
            PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)).ok();
        }
        // wait for it to cleanup
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

/// Callback for EnumWindows to seed the initial cache.
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
    trace!("enum_windows_proc: {:?}", hwnd);
    if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return BOOL(1);
    }

    // get rect
    let rect = {
        let mut r = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut r) }.is_ok() {
            Some(r.as_bevy_irect())
        } else {
            None
        }
    };

    // get title
    let title = {
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
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
            rect: rect,
            title,
            timestamp: Local::now(),
        },
    );
    BOOL(1) // continue
}

/// WinEvent callback: updates or removes an entry on move/resize/title‐change.
unsafe extern "system" fn win_event_proc(
    _: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _: u32,
    _: u32,
) {
    trace!(
        "win_event_proc: hwnd={:?}, event={}, obj={}, child={}",
        hwnd, event, id_object, id_child
    );

    // only top‐level window changes
    if hwnd.is_invalid() || id_object != 0 || id_child != 0 {
        return;
    }
    if event != EVENT_OBJECT_LOCATIONCHANGE && event != EVENT_OBJECT_NAMECHANGE {
        return;
    }

    // if now invisible, drop it
    if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        WINDOWS.lock().unwrap().remove(&(hwnd.0 as isize));
        return;
    }

    // otherwise requery and update
    let rect = {
        let mut r = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut r) }.is_ok() {
            Some(r.as_bevy_irect())
        } else {
            None
        }
    };
    let title = {
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
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
