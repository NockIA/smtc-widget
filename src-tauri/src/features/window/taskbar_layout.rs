use tauri::Monitor;

use super::constants::WEATHER_INNER_PADDING;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetClassNameW, GetParent, GetWindowRect, IsWindowVisible,
};
use windows::core::BOOL;

const TASKBAR_CLASSES: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];
/// Conteneur des boutons d'applications (barre principale / barres secondaires).
const TASK_LIST_CLASSES: [&str; 2] = ["ReBarWindow32", "WorkerW"];
/// Widget météo de Windows 10 (DynamicContent1, DynamicContent2…).
const WEATHER_CLASS_PREFIX: &str = "DynamicContent";

struct Child {
    class: String,
    rect: RECT,
}

fn class_name(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn window_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
    Some(rect)
}

unsafe extern "system" fn collect_taskbar(hwnd: HWND, list: LPARAM) -> BOOL {
    let list = unsafe { &mut *(list.0 as *mut Vec<HWND>) };
    if TASKBAR_CLASSES.contains(&class_name(hwnd).as_str()) {
        list.push(hwnd);
    }
    true.into()
}

unsafe extern "system" fn collect_child(hwnd: HWND, list: LPARAM) -> BOOL {
    let list = unsafe { &mut *(list.0 as *mut Vec<HWND>) };
    list.push(hwnd);
    true.into()
}

fn taskbars() -> Vec<HWND> {
    let mut list: Vec<HWND> = Vec::new();
    let _ = unsafe { EnumWindows(Some(collect_taskbar), LPARAM(&mut list as *mut _ as isize)) };
    list
}

/// Enfants directs et visibles de la barre (les descendants sont ignorés).
fn direct_children(taskbar: HWND) -> Vec<Child> {
    let mut all: Vec<HWND> = Vec::new();
    let _ = unsafe { EnumChildWindows(Some(taskbar), Some(collect_child), LPARAM(&mut all as *mut _ as isize)) };
    all.into_iter()
        .filter(|&hwnd| unsafe { GetParent(hwnd) }.is_ok_and(|parent| parent == taskbar))
        .filter(|&hwnd| unsafe { IsWindowVisible(hwnd) }.as_bool())
        .filter_map(|hwnd| {
            Some(Child {
                class: class_name(hwnd),
                rect: window_rect(hwnd)?,
            })
        })
        .collect()
}

fn on_monitor(rect: &RECT, monitor: &Monitor) -> bool {
    let center_x = (rect.left + rect.right) / 2;
    let center_y = (rect.top + rect.bottom) / 2;
    let position = monitor.position();
    let size = monitor.size();
    center_x >= position.x
        && center_x < position.x + size.width as i32
        && center_y >= position.y
        && center_y < position.y + size.height as i32
}

/// Bord gauche (pixels physiques) des éléments à droite de la barre des tâches de cet écran
/// (météo, zone de notification, horloge). `None` si la barre n'est pas lisible : absente,
/// masquée automatiquement, verticale, ou structure inconnue (Windows 11).
pub fn right_cluster_left(monitor: &Monitor) -> Option<i32> {
    let taskbar = taskbars().into_iter().find(|&hwnd| {
        window_rect(hwnd)
            .is_some_and(|rect| on_monitor(&rect, monitor) && rect.right - rect.left > rect.bottom - rect.top)
    })?;

    let children = direct_children(taskbar);
    let task_list_right = children
        .iter()
        .find(|child| TASK_LIST_CLASSES.contains(&child.class.as_str()))?
        .rect
        .right;

    children
        .iter()
        .filter(|child| child.rect.left >= task_list_right && child.rect.right > child.rect.left)
        .map(|child| {
            // La fenêtre météo garde une marge vide à gauche de son icône.
            if child.class.starts_with(WEATHER_CLASS_PREFIX) {
                child.rect.left + (WEATHER_INNER_PADDING * monitor.scale_factor()) as i32
            } else {
                child.rect.left
            }
        })
        .min()
}
