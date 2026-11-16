use tauri::Monitor;
use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
use windows::Win32::UI::WindowsAndMessaging::{EnumChildWindows, EnumWindows, GetParent, IsWindowVisible};
use windows::core::BOOL;

use super::win32::{get_window_class_name, get_window_screen_rect};

/// Marge vide (pixels logiques) entre le bord gauche de la fenêtre météo et son icône.
const WEATHER_INNER_PADDING: f64 = 12.0;

const TASKBAR_CLASSES: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];
/// Conteneur des boutons d'applications (barre principale / barres secondaires).
const TASK_LIST_CLASSES: [&str; 2] = ["ReBarWindow32", "WorkerW"];
/// Widget météo de Windows 10 (`DynamicContent1`, `DynamicContent2`…).
const WEATHER_CLASS_PREFIX: &str = "DynamicContent";

const MIN_PLAUSIBLE_HEIGHT: f64 = 24.0;
const MAX_PLAUSIBLE_HEIGHT: f64 = 120.0;

struct Child {
    class: String,
    rect: RECT,
}

unsafe extern "system" fn collect_taskbar(hwnd: HWND, list: LPARAM) -> BOOL {
    let list = unsafe { &mut *(list.0 as *mut Vec<HWND>) };
    if TASKBAR_CLASSES.contains(&get_window_class_name(hwnd).as_str()) {
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
    let _ = unsafe { EnumWindows(Some(collect_taskbar), LPARAM(std::ptr::from_mut(&mut list) as isize)) };
    list
}

/// Enfants directs et visibles de la barre (les descendants sont ignorés).
fn direct_children(taskbar: HWND) -> Vec<Child> {
    let mut all: Vec<HWND> = Vec::new();
    let _ = unsafe {
        EnumChildWindows(
            Some(taskbar),
            Some(collect_child),
            LPARAM(std::ptr::from_mut(&mut all) as isize),
        )
    };
    all.into_iter()
        .filter(|&hwnd| unsafe { GetParent(hwnd) }.is_ok_and(|parent| parent == taskbar))
        .filter(|&hwnd| unsafe { IsWindowVisible(hwnd) }.as_bool())
        .filter_map(|hwnd| {
            Some(Child {
                class: get_window_class_name(hwnd),
                rect: get_window_screen_rect(hwnd)?,
            })
        })
        .collect()
}

fn is_on_monitor(rect: &RECT, monitor: &Monitor) -> bool {
    let center_x = i32::midpoint(rect.left, rect.right);
    let center_y = i32::midpoint(rect.top, rect.bottom);
    let position = monitor.position();
    let size = monitor.size();
    center_x >= position.x
        && center_x < position.x + size.width as i32
        && center_y >= position.y
        && center_y < position.y + size.height as i32
}

/// Barre des tâches horizontale de cet écran.
fn horizontal_taskbar(monitor: &Monitor) -> Option<HWND> {
    taskbars().into_iter().find(|&hwnd| {
        get_window_screen_rect(hwnd)
            .is_some_and(|rect| is_on_monitor(&rect, monitor) && rect.right - rect.left > rect.bottom - rect.top)
    })
}

/// Numéro de build à partir duquel Windows est la version 11.
const WINDOWS_11_FIRST_BUILD: u32 = 22000;

/// Hauteur par défaut de la barre des tâches (pixels logiques) selon la version de Windows : 48 sous Windows 11, 40 sous Windows 10.
fn default_taskbar_height() -> f64 {
    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    // `RtlGetVersion` donne la vraie version, contrairement à `GetVersionEx` qui dépend du manifeste.
    let is_windows_11 = unsafe { RtlGetVersion(&mut info) }.is_ok() && info.dwBuildNumber >= WINDOWS_11_FIRST_BUILD;
    if is_windows_11 { 48.0 } else { 40.0 }
}

/// Hauteur lue de la barre des tâches de cet écran (pixels logiques), si elle est plausible :
/// une barre masquée automatiquement n'a presque plus de hauteur visible.
fn measured_taskbar_height(monitor: &Monitor) -> Option<f64> {
    let rect = get_window_screen_rect(horizontal_taskbar(monitor)?)?;
    let height = f64::from(rect.bottom - rect.top) / monitor.scale_factor();
    (MIN_PLAUSIBLE_HEIGHT..=MAX_PLAUSIBLE_HEIGHT)
        .contains(&height)
        .then_some(height)
}

/// Hauteur de la barre des tâches de cet écran (pixels logiques) : lue directement, sinon celle par défaut de Windows.
pub fn taskbar_height(monitor: &Monitor) -> f64 {
    measured_taskbar_height(monitor).unwrap_or_else(default_taskbar_height)
}

/// Cherche où commence, en pixels physiques, la zone système à droite de la barre des tâches (météo, notifications, horloge).
/// Pour cela, elle prend la barre horizontale de l'écran, puis le bord gauche le plus à gauche de ses éléments
/// situés après la liste des applications.
/// `None` si la barre n'est pas lisible (masquée, verticale, structure inconnue…).
pub fn find_system_area_left_edge(monitor: &Monitor) -> Option<i32> {
    let taskbar = horizontal_taskbar(monitor)?;

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
