//! Attach a window behind the desktop icons (Progman / WorkerW trick).

use anyhow::{Context, Result, anyhow};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GWL_STYLE, GetCursorPos, GetSystemMetrics,
    GetWindowLongPtrW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SMTO_NORMAL, SWP_NOACTIVATE,
    SWP_SHOWWINDOW, SendMessageTimeoutW, SetParent, SetWindowLongPtrW, SetWindowPos, WS_CHILD,
    WS_POPUP,
};
use windows::core::{BOOL, w};

pub fn set_dpi_aware() {
    unsafe {
        let _ = SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
    }
}

/// Size of the whole virtual desktop (all monitors) in physical pixels.
pub fn virtual_screen_size() -> (i32, i32) {
    unsafe {
        (
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    }
}

/// Cursor position in screen coordinates.
pub fn cursor_pos() -> Option<(i32, i32)> {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p).ok()? };
    Some((p.x, p.y))
}

unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        // The top-level window that hosts the icons (SHELLDLL_DefView)...
        let defview = FindWindowExW(Some(hwnd), None, w!("SHELLDLL_DefView"), None);
        if defview.is_ok() {
            // ...is followed by the WorkerW we want to draw into.
            if let Ok(worker) = FindWindowExW(None, Some(hwnd), w!("WorkerW"), None) {
                *(lparam.0 as *mut HWND) = worker;
            }
        }
    }
    BOOL(1)
}

fn find_worker_w() -> Result<HWND> {
    unsafe {
        let progman = FindWindowW(w!("Progman"), None).context("Progman not found")?;

        // Ask Explorer to spawn a WorkerW behind the icons.
        let mut result = 0usize;
        SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0xD),
            LPARAM(0x1),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        );

        let mut worker = HWND::default();
        let _ = EnumWindows(Some(enum_cb), LPARAM(&mut worker as *mut HWND as isize));
        if !worker.0.is_null() {
            return Ok(worker);
        }

        // Newer Windows 11 builds parent the WorkerW under Progman instead.
        FindWindowExW(Some(progman), None, w!("WorkerW"), None)
            .map_err(|_| anyhow!("WorkerW not found"))
    }
}

/// Reparent `hwnd` under the desktop WorkerW and stretch it over the virtual desktop.
pub fn attach_to_desktop(hwnd: HWND) -> Result<()> {
    let worker = find_worker_w()?;
    let (w, h) = virtual_screen_size();
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let style = (style & !WS_POPUP.0) | WS_CHILD.0;
        SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize);
        SetParent(hwnd, Some(worker)).context("SetParent failed")?;
        SetWindowPos(hwnd, None, 0, 0, w, h, SWP_NOACTIVATE | SWP_SHOWWINDOW)
            .context("SetWindowPos failed")?;
    }
    Ok(())
}
