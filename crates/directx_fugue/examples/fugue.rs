use directx_fugue::init_dx11;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::Graphics::Dxgi::DXGI_PRESENT;
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::DispatchMessageW;
use windows::Win32::UI::WindowsAndMessaging::MSG;
use windows::Win32::UI::WindowsAndMessaging::PM_REMOVE;
use windows::Win32::UI::WindowsAndMessaging::PeekMessageW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::TranslateMessage;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;
// … other imports …

static CLASS_NAME: &str = "MyWindowClass";

fn main() -> eyre::Result<()> {
    // 1) create your window, register class, etc…
    let hwnd: HWND = unsafe {
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
            // WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT,
            WS_EX_TOPMOST | WS_EX_TRANSPARENT,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wnd_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            // WS_POPUPWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            600,
            None,            // no parent
            None,            // no menu
            Some(hinstance), // our HINSTANCE
            None,            // no extra param
        )?; // propagate Err if window creation fails
        hwnd
    };

    // 2) actually show it
    unsafe {
        _ = ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd).ok()?;
    }
    tracing::info!("Created window: {:?}", hwnd);

    // 3) initialize D3D11
    let (_device, context, swapchain, rtv) = init_dx11(hwnd)?;
    tracing::info!("DirectX 11 initialized!");

    // 4) main loop: use PeekMessage to avoid blocking
    let mut msg = MSG::default();
    loop {
        // pump all pending messages
        while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.into() {
            if msg.message == WM_QUIT {
                return Ok(());
            }
            unsafe {
                _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        // clear to solid red
        unsafe {
            context.ClearRenderTargetView(&rtv, &[1.0, 0.0, 0.0, 0.5]);
        }

        // flip buffers
        unsafe { swapchain.Present(1, DXGI_PRESENT::default()) }.ok()?;
    }
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
