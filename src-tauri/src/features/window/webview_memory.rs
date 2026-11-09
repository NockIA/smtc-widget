use tauri::WebviewWindow;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL, ICoreWebView2_19,
};
use windows::core::Interface;

/// Demande à `WebView2` de libérer sa mémoire tant que le widget est caché.
/// Sans effet sur un runtime `WebView2` trop ancien pour cette option.
pub fn set_low_memory(window: &WebviewWindow, low: bool) {
    let level = if low {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };

    let result = window.with_webview(move |webview| unsafe {
        let Ok(core) = webview.controller().CoreWebView2() else {
            return;
        };
        if let Ok(core) = core.cast::<ICoreWebView2_19>() {
            let _ = core.SetMemoryUsageTargetLevel(level);
        }
    });
    if let Err(e) = result {
        log::error!("réglage mémoire de WebView2 a échoué : {e}");
    }
}
