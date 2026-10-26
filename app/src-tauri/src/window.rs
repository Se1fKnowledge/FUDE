//! Showing the editor inside the FUDE window.
//!
//! PhotoCraft stays its own process (its rules forbid wrapping it in a webview). On Windows its
//! top-level window is reparented into FUDE's window as a borderless child and placed over the
//! rectangle the interface reserves for it; detaching restores it as a normal window. This is
//! the only module with `unsafe` code: thin, checked calls into user32.

/// A rectangle in physical pixels of the FUDE window's client area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn is_usable(&self) -> bool {
        self.width >= 50 && self.height >= 50
    }
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod imp {
    use super::Rect;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GW_OWNER, GWL_EXSTYLE, GWL_STYLE, GetWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
        SW_HIDE, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW, SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, WS_CAPTION,
        WS_CHILD, WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_EX_DLGMODALFRAME, WS_EX_WINDOWEDGE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
    };

    fn hwnd(raw: isize) -> HWND {
        raw as HWND
    }

    struct Search {
        pid: u32,
        best: isize,
        area: i64,
    }

    unsafe extern "system" fn visit(window: HWND, data: LPARAM) -> i32 {
        // SAFETY: `data` is the `&mut Search` passed to `EnumWindows` below, alive for the call.
        let search = unsafe { &mut *(data as *mut Search) };
        let mut pid = 0u32;
        // SAFETY: `window` comes from EnumWindows; `pid` is a valid out pointer.
        unsafe { GetWindowThreadProcessId(window, &mut pid) };
        // SAFETY: plain queries on a window handle EnumWindows just gave us.
        let candidate = pid == search.pid && unsafe { IsWindowVisible(window) } != 0 && unsafe { GetWindow(window, GW_OWNER) }.is_null();
        if candidate {
            let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            // SAFETY: `r` is a valid out pointer.
            if unsafe { GetWindowRect(window, &mut r) } != 0 {
                let area = i64::from(r.right - r.left) * i64::from(r.bottom - r.top);
                if area > search.area {
                    search.area = area;
                    search.best = window as isize;
                }
            }
        }
        1
    }

    /// The editor's main window: its largest visible, unowned top-level window.
    pub fn main_window(pid: u32) -> Option<isize> {
        let mut search = Search { pid, best: 0, area: 0 };
        // SAFETY: the callback only reads through the pointer to `search`, which outlives the call.
        unsafe { EnumWindows(Some(visit), &mut search as *mut Search as LPARAM) };
        (search.best != 0).then_some(search.best)
    }

    pub fn alive(window: isize) -> bool {
        // SAFETY: IsWindow accepts any value and only reports whether it is a window.
        unsafe { IsWindow(hwnd(window)) != 0 }
    }

    /// Window styles to restore when the editor is detached again.
    #[derive(Debug, Clone, Copy)]
    pub struct Saved {
        style: isize,
        ex_style: isize,
        rect: Rect,
    }

    pub fn embed(window: isize, host: isize, rect: Rect) -> Saved {
        let w = hwnd(window);
        // SAFETY: style queries and changes on the editor's window, which `alive` was checked for
        // by the caller; SetParent to FUDE's own window.
        unsafe {
            let style = GetWindowLongPtrW(w, GWL_STYLE);
            let ex_style = GetWindowLongPtrW(w, GWL_EXSTYLE);
            let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetWindowRect(w, &mut r);
            let frame = (WS_POPUP | WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX) as isize;
            SetWindowLongPtrW(w, GWL_STYLE, (style & !frame) | (WS_CHILD | WS_CLIPSIBLINGS) as isize);
            SetWindowLongPtrW(w, GWL_EXSTYLE, ex_style & !((WS_EX_APPWINDOW | WS_EX_WINDOWEDGE | WS_EX_DLGMODALFRAME) as isize));
            SetParent(w, hwnd(host));
            place(window, rect);
            Saved { style, ex_style, rect: Rect { x: r.left, y: r.top, width: r.right - r.left, height: r.bottom - r.top } }
        }
    }

    /// Moves the embedded editor over `rect` and above FUDE's webview.
    pub fn place(window: isize, rect: Rect) {
        // SAFETY: SetWindowPos on the editor's window; HWND_TOP is the null handle.
        unsafe {
            SetWindowPos(hwnd(window), std::ptr::null_mut(), rect.x, rect.y, rect.width, rect.height, SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW);
        }
    }

    pub fn show(window: isize, visible: bool) {
        // SAFETY: ShowWindow on the editor's window.
        unsafe { ShowWindow(hwnd(window), if visible { SW_SHOW } else { SW_HIDE }) };
    }

    /// Makes the editor a normal top-level window again, where it was before.
    pub fn detach(window: isize, saved: Saved) {
        let w = hwnd(window);
        // SAFETY: restores what `embed` read from this same window.
        unsafe {
            SetParent(w, std::ptr::null_mut());
            SetWindowLongPtrW(w, GWL_STYLE, saved.style);
            SetWindowLongPtrW(w, GWL_EXSTYLE, saved.ex_style);
            let r = saved.rect;
            SetWindowPos(w, std::ptr::null_mut(), r.x, r.y, r.width.max(800), r.height.max(600), SWP_NOZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW);
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Rect;

    #[derive(Debug, Clone, Copy)]
    pub struct Saved;

    pub fn main_window(_pid: u32) -> Option<isize> {
        None
    }
    pub fn alive(_window: isize) -> bool {
        false
    }
    pub fn embed(_window: isize, _host: isize, _rect: Rect) -> Saved {
        Saved
    }
    pub fn place(_window: isize, _rect: Rect) {}
    pub fn show(_window: isize, _visible: bool) {}
    pub fn detach(_window: isize, _saved: Saved) {}
}

pub use imp::{Saved, alive, detach, embed, main_window, place, show};

/// Whether the editor can be shown inside the FUDE window on this system.
pub const EMBEDDING_SUPPORTED: bool = cfg!(windows);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_rectangles_are_not_used() {
        assert!(Rect { x: 0, y: 0, width: 800, height: 600 }.is_usable());
        assert!(!Rect { x: 0, y: 0, width: 0, height: 600 }.is_usable());
        assert!(!Rect { x: 10, y: 10, width: 49, height: 49 }.is_usable());
    }

    #[test]
    fn a_process_without_windows_has_no_main_window() {
        // This test process opens no windows.
        assert_eq!(main_window(std::process::id()), None);
        assert!(!alive(0));
    }
}
