use color_eyre::eyre::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

static CLASS_NAME: &str = "MyWindowClass";

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_line_number(true)
        .with_file(true)
        .with_max_level(tracing::level_filters::LevelFilter::DEBUG)
        .init();

    unsafe {
        // GetModuleHandleW returns HMODULE; we need an HINSTANCE for WNDCLASSW
        let hmodule = GetModuleHandleW(None)?;
        let hinstance = hmodule.into(); // Convert HMODULE -> HINSTANCE

        let class_name = widestring::U16CString::from_str(CLASS_NAME)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassW(&wc);

        // CreateWindowExW wants Option<HWND>, Option<HMENU>, Option<HINSTANCE>, Option<*const c_void>
        let wnd_name = widestring::U16CString::from_str("DX11 Window")?;
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wnd_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            600,
            None,            // no parent
            None,            // no menu
            Some(hinstance), // our HINSTANCE
            None,            // no extra param
        )?; // propagate Err if window creation fails

        _ = ShowWindow(hwnd, SW_SHOW);
        tracing::info!("Created window: {:?}", hwnd);

        let (device, context, swapchain, rtv) = directx_fugue::init_dx11(hwnd)?;
        tracing::info!("DirectX 11 initialized!");

        // Basic message loop: pass None to listen for all messages
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up in reverse order
        drop(rtv);
        drop(swapchain);
        drop(context);
        drop(device);
    }

    Ok(())
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
