use std::ffi::c_void;

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};
use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetWindowRect};

pub fn class_name(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

pub fn window_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
    Some(rect)
}

/// Rectangle complet de l'écran (barre des tâches comprise), en pixels physiques.
pub fn monitor_rect(monitor: HMONITOR) -> Option<RECT> {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetMonitorInfoW(monitor, &mut info) }
        .as_bool()
        .then_some(info.rcMonitor)
}

/// Fenêtre « cloaked » par DWM : techniquement visible mais non affichée (application UWP en veille, autre bureau virtuel).
pub fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            (&raw mut cloaked).cast::<c_void>(),
            size_of::<u32>() as u32,
        )
    };
    result.is_ok() && cloaked != 0
}
