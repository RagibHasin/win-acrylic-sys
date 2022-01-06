// #![windows_subsystem = "windows"]

use std::ptr::{null, null_mut};

use windows::{
    core::Result,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, PWSTR, WPARAM},
        Graphics::Gdi::ValidateRect,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

use win_acrylic_sys as acrylic;

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None);
        debug_assert!(instance != 0);

        let window_class: Vec<u16> = b"window\0".iter().copied().map(Into::into).collect();

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW),
            hInstance: instance,
            lpszClassName: PWSTR(window_class.as_ptr() as _),

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            WS_EX_NOREDIRECTIONBITMAP,
            PWSTR(window_class.as_ptr() as _),
            "This is a sample window with acrylic background",
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
            None,
            None,
            instance,
            null_mut(),
        );

        acrylic::set_effect(
            hwnd,
            acrylic::Effect::Acrylic,
            true,
            Some((0x20, 0x20, 0x20, 0x20)),
        );

        let mut message = Default::default();

        while GetMessageW(&mut message, 0, 0, 0).into() {
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg as u32 {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(hwnd, null());
                0
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
