use windows::Win32::Media::Audio::{
    IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
    eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize};
use windows::core::Interface;

use super::process;
use crate::shared::smtc::{active_session, source_id};

/// Évite les faux positifs avec des noms d'exécutable très courts.
const MIN_EXE_STEM_LEN: usize = 3;

struct ComGuard(bool);

impl ComGuard {
    fn new() -> Self {
        // `is_ok` couvre S_OK et S_FALSE (déjà initialisé) : les deux demandent un `CoUninitialize`.
        // En cas d'échec (ex : thread déjà en STA), COM reste utilisable et on ne libère rien.
        Self(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok())
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

/// Vrai si l'exécutable correspond à la source SMTC : nom identique
/// ("Spotify.exe") ou nom contenu dans l'identifiant (paquet du Store, "Chrome", etc.).
fn source_matches(source: &str, exe: &str) -> bool {
    let source = source.to_lowercase();
    let exe = exe.to_lowercase();
    if source == exe {
        return true;
    }
    let stem = exe.strip_suffix(".exe").unwrap_or(&exe);
    stem.len() >= MIN_EXE_STEM_LEN && source.contains(stem)
}

fn source_volumes(source: &str) -> windows::core::Result<Vec<ISimpleAudioVolume>> {
    unsafe {
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
        let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
        let sessions = manager.GetSessionEnumerator()?;

        let mut volumes = Vec::new();
        for index in 0..sessions.GetCount()? {
            let control = sessions.GetSession(index)?;
            let pid = control.cast::<IAudioSessionControl2>()?.GetProcessId()?;
            if pid == 0 {
                continue;
            }
            let Some(exe) = process::exe_name(pid) else {
                continue;
            };
            if source_matches(source, &exe) {
                volumes.push(control.cast::<ISimpleAudioVolume>()?);
            }
        }
        Ok(volumes)
    }
}

fn current_source(selected: Option<&str>) -> windows::core::Result<Option<String>> {
    Ok(active_session(selected)?.map(|session| source_id(&session)))
}

/// Volume (0.0 à 1.0) de la source courante, `None` si aucune session audio ne correspond.
pub fn get_volume(selected: Option<&str>) -> windows::core::Result<Option<f32>> {
    let Some(source) = current_source(selected)? else {
        return Ok(None);
    };
    let _com = ComGuard::new();

    let mut volume: Option<f32> = None;
    for control in source_volumes(&source)? {
        let level = unsafe { control.GetMasterVolume()? };
        volume = Some(volume.map_or(level, |v| v.max(level)));
    }
    Ok(volume)
}

/// Renvoie `false` si aucune session audio ne correspond.
pub fn set_volume(selected: Option<&str>, volume: f32) -> windows::core::Result<bool> {
    let Some(source) = current_source(selected)? else {
        return Ok(false);
    };
    let _com = ComGuard::new();

    let controls = source_volumes(&source)?;
    for control in &controls {
        unsafe { control.SetMasterVolume(volume.clamp(0.0, 1.0), std::ptr::null())? };
    }
    Ok(!controls.is_empty())
}
