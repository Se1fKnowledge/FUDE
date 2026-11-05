//! Showing the editor inside the FUDE window.
//!
//! PhotoCraft stays its own process (its rules forbid wrapping it in a webview). On Windows its
//! top-level window is reparented into FUDE's window as a borderless child and placed over the
//! rectangle the interface reserves for it; detaching restores it as a normal window. This is
//! the only module with `unsafe` code: thin, checked calls into user32.
//!
//! A child window of another process does not get the keyboard by itself: clicks in it reach
//! FUDE's window as `WM_PARENTNOTIFY`, and FUDE then gives it the focus. The hint shortcut (F1)
//! is a hot key registered while FUDE's window is active, so it works with the focus inside the
//! embedded editor too, and is released as soon as another window comes forward.

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

    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use windows_sys::Win32::Foundation::{LRESULT, WPARAM};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, RegisterHotKey, SetFocus, UnregisterHotKey, VK_F1};
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, PostMessageW, WA_INACTIVE, WM_ACTIVATE, WM_APP, WM_HOTKEY, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_PARENTNOTIFY, WM_POINTERDOWN,
        WM_RBUTTONDOWN, WM_XBUTTONDOWN,
    };

    fn hwnd(raw: isize) -> HWND {
        raw as HWND
    }

    /// FUDE's main window, the embedded editor's window (0: none) and whether the hint key is
    /// wanted (a practice is open).
    static HOST: AtomicIsize = AtomicIsize::new(0);
    static EDITOR: AtomicIsize = AtomicIsize::new(0);
    static HINT_KEY: AtomicBool = AtomicBool::new(false);
    static ON_HINT: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
    /// Posted to FUDE's window: give the embedded editor the keyboard / re-check the hot key.
    const FOCUS_EDITOR: u32 = WM_APP + 0x46;
    const SYNC_HOTKEY: u32 = WM_APP + 0x47;
    const HOTKEY_ID: i32 = 0x4655;

    /// A press of any mouse button or pen in a child window, as `WM_PARENTNOTIFY` reports it.
    pub fn is_press(event: u32) -> bool {
        matches!(event, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN | WM_POINTERDOWN)
    }

    fn focus_editor() {
        let editor = EDITOR.load(Ordering::SeqCst);
        if editor != 0 && alive(editor) {
            // SAFETY: runs on FUDE's window thread, whose input the child editor window shares.
            unsafe { SetFocus(hwnd(editor)) };
        }
    }

    fn sync_hotkey(host: HWND, active: bool) {
        // SAFETY: hot keys of FUDE's own window, on its thread; unregistering a missing one is a no-op.
        unsafe {
            UnregisterHotKey(host, HOTKEY_ID);
            if active && HINT_KEY.load(Ordering::SeqCst) {
                RegisterHotKey(host, HOTKEY_ID, MOD_NOREPEAT, u32::from(VK_F1));
            }
        }
    }

    unsafe extern "system" fn host_proc(window: HWND, msg: u32, wparam: WPARAM, lparam: isize, _id: usize, _data: usize) -> LRESULT {
        match msg {
            WM_PARENTNOTIFY if is_press((wparam & 0xffff) as u32) => focus_editor(),
            FOCUS_EDITOR => focus_editor(),
            WM_ACTIVATE => sync_hotkey(window, (wparam & 0xffff) as u32 != WA_INACTIVE),
            // SAFETY: a plain query.
            SYNC_HOTKEY => sync_hotkey(window, unsafe { GetForegroundWindow() } == window),
            WM_HOTKEY if wparam as i32 == HOTKEY_ID => {
                if let Some(on_hint) = ON_HINT.get() {
                    on_hint();
                }
                return 0;
            }
            _ => {}
        }
        // SAFETY: hands every message on to the window's own procedure.
        unsafe { DefSubclassProc(window, msg, wparam, lparam) }
    }

    /// Watches FUDE's main window for clicks in the embedded editor and for the hint hot key.
    /// Call once, on the window's own thread.
    pub fn install_host(host: isize, on_hint: Box<dyn Fn() + Send + Sync>) {
        if host == 0 {
            return;
        }
        HOST.store(host, Ordering::SeqCst);
        let _ = ON_HINT.set(on_hint);
        // SAFETY: subclasses FUDE's own window from its own thread; `host_proc` lives forever.
        unsafe { SetWindowSubclass(hwnd(host), Some(host_proc), 1, 0) };
    }

    fn post(msg: u32) {
        let host = HOST.load(Ordering::SeqCst);
        if host != 0 {
            // SAFETY: posting to FUDE's own window; it handles the message on its thread.
            unsafe { PostMessageW(hwnd(host), msg, 0, 0) };
        }
    }

    /// Turns the F1 hint key on (a practice is open) or off.
    pub fn set_hint_key(enabled: bool) {
        HINT_KEY.store(enabled, Ordering::SeqCst);
        post(SYNC_HOTKEY);
    }

    /// Gives the embedded editor the keyboard.
    pub fn focus_editor_soon() {
        post(FOCUS_EDITOR);
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
        let candidate = pid == search.pid
            && unsafe { IsWindowVisible(window) } != 0
            && unsafe { GetWindow(window, GW_OWNER) }.is_null()
            && super::is_app_window(unsafe { GetWindowLongPtrW(window, GWL_EXSTYLE) } as u32);
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
            EDITOR.store(window, Ordering::SeqCst);
            post(FOCUS_EDITOR);
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
        EDITOR.store(0, Ordering::SeqCst);
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
    pub fn install_host(_host: isize, _on_hint: Box<dyn Fn() + Send + Sync>) {}
    pub fn set_hint_key(_enabled: bool) {}
    pub fn focus_editor_soon() {}
}

pub use imp::{Saved, alive, detach, embed, focus_editor_soon, install_host, main_window, place, set_hint_key, show};

/// Whether a window with these extended styles can be the editor's main window. winit keeps a
/// hidden «Winit Thread Event Target» window per thread: a transparent, non-activating tool window
/// that counts as visible. Embedding it instead of the real window left the editor floating on
/// its own and the keyboard nowhere.
pub fn is_app_window(ex_style: u32) -> bool {
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
    const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
    ex_style & (WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE) == 0
}

/// Whether the editor can be shown inside the FUDE window on this system.
pub const EMBEDDING_SUPPORTED: bool = cfg!(windows);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_windows_are_never_taken_for_the_editor() {
        // winit's event target as found in a live PhotoCraft: layered, transparent, tool, no-activate.
        assert!(!is_app_window(0x0808_00A0));
        assert!(!is_app_window(0x80), "a tool window");
        assert!(!is_app_window(0x0800_0000), "a window that never activates");
        // The real editor window: WS_EX_WINDOWEDGE | WS_EX_APPWINDOW | WS_EX_CONTROLPARENT.
        assert!(is_app_window(0x0004_0110));
        assert!(is_app_window(0));
    }

    #[test]
    fn tiny_rectangles_are_not_used() {
        assert!(Rect { x: 0, y: 0, width: 800, height: 600 }.is_usable());
        assert!(!Rect { x: 0, y: 0, width: 0, height: 600 }.is_usable());
        assert!(!Rect { x: 10, y: 10, width: 49, height: 49 }.is_usable());
    }

    #[cfg(windows)]
    #[test]
    fn presses_of_every_button_and_the_pen_hand_the_editor_the_keyboard() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_POINTERDOWN, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
        };
        for press in [WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN, WM_XBUTTONDOWN, WM_POINTERDOWN] {
            assert!(imp::is_press(press), "{press:#x}");
        }
        // A child window opening or closing is not a click.
        assert!(!imp::is_press(WM_CREATE));
        assert!(!imp::is_press(WM_DESTROY));
    }

    #[test]
    fn hint_key_and_focus_calls_are_safe_without_a_window() {
        // Before the main window is watched, these must quietly do nothing.
        set_hint_key(true);
        set_hint_key(false);
        focus_editor_soon();
    }

    #[test]
    fn a_process_without_windows_has_no_main_window() {
        // This test process opens no windows.
        assert_eq!(main_window(std::process::id()), None);
        assert!(!alive(0));
    }
}
