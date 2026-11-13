use std::collections::HashMap;

use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo,
    GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS};

/// Préfixe du nom Windows d'un écran (`\\.\DISPLAY2`), dont le numéro est celui affiché dans Paramètres › Affichage.
const DISPLAY_NAME_PREFIX: &str = r"\\.\DISPLAY";

/// Nombre d'essais si la liste des écrans change entre la lecture de sa taille et celle de son contenu.
const QUERY_ATTEMPTS: u32 = 3;

/// Numéro de l'écran dans Paramètres › Affichage, déduit de son nom Windows (`\\.\DISPLAY2` donne 2).
pub fn get_display_number(display_name: &str) -> Option<u32> {
    display_name.strip_prefix(DISPLAY_NAME_PREFIX)?.parse().ok()
}

/// Noms commerciaux des écrans actifs (ex : `DELL U2419H`), indexés par leur nom Windows (`\\.\DISPLAY2`).
/// Un écran sans nom lisible est absent de la table.
pub fn get_friendly_names() -> HashMap<String, String> {
    query_active_paths()
        .iter()
        .filter_map(|path| {
            let display_name = get_source_name(path)?;
            let friendly_name = get_target_name(path).filter(|name| !name.is_empty())?;
            Some((display_name, friendly_name))
        })
        .collect()
}

/// Liste les liens « carte graphique vers écran » actuellement actifs.
fn query_active_paths() -> Vec<DISPLAYCONFIG_PATH_INFO> {
    for _ in 0..QUERY_ATTEMPTS {
        let mut path_count = 0;
        let mut mode_count = 0;
        let sizes = unsafe { GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count) };
        if sizes != ERROR_SUCCESS {
            break;
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
        let result = unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            )
        };
        if result == ERROR_SUCCESS {
            paths.truncate(path_count as usize);
            return paths;
        }
        if result != ERROR_INSUFFICIENT_BUFFER {
            break;
        }
    }
    Vec::new()
}

/// Nom Windows de l'écran (`\\.\DISPLAY2`) pour ce lien.
fn get_source_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
    let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
            size: size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
            adapterId: path.sourceInfo.adapterId,
            id: path.sourceInfo.id,
        },
        ..Default::default()
    };
    // Windows remplit toute la structure à partir de son en-tête : on lui passe donc un pointeur sur l'ensemble.
    if unsafe { DisplayConfigGetDeviceInfo((&raw mut source).cast()) } != 0 {
        return None;
    }
    Some(utf16_to_string(&source.viewGdiDeviceName))
}

/// Nom commercial du moniteur branché sur ce lien.
fn get_target_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
    let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            size: size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
            adapterId: path.targetInfo.adapterId,
            id: path.targetInfo.id,
        },
        ..Default::default()
    };
    if unsafe { DisplayConfigGetDeviceInfo((&raw mut target).cast()) } != 0 {
        return None;
    }
    Some(utf16_to_string(&target.monitorFriendlyDeviceName))
}

/// Texte d'un tampon Windows, qui se termine au premier caractère nul.
fn utf16_to_string(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}
