use windows::Win32::Media::Audio::{
    IAudioSessionControl, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume,
    MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize};
use windows::core::{GUID, Interface};

use super::process;

/// Évite les faux positifs avec des noms d'exécutable très courts.
const MIN_EXE_STEM_LEN: usize = 3;

/// Contexte passé à `SetMasterVolume` : la surveillance ignore ainsi les changements faits par le widget lui-même.
pub(super) const WIDGET_CONTEXT: GUID = GUID::from_u128(0x6d0c_2f4e_8b1a_4c7e_9a35_1f2b_7c8d_9e01);

pub(super) struct ComGuard(bool);

impl ComGuard {
    pub(super) fn new() -> Self {
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
fn is_source_matching(source: &str, exe: &str) -> bool {
    let source = source.to_lowercase();
    let exe = exe.to_lowercase();
    if source == exe {
        return true;
    }
    let stem = exe.strip_suffix(".exe").unwrap_or(&exe);
    stem.len() >= MIN_EXE_STEM_LEN && source.contains(stem)
}

pub(super) fn device_enumerator() -> windows::core::Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
}

/// Gestionnaire des sessions audio du périphérique de sortie par défaut.
pub(super) fn session_manager(enumerator: &IMMDeviceEnumerator) -> windows::core::Result<IAudioSessionManager2> {
    unsafe {
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
        device.Activate(CLSCTX_ALL, None)
    }
}

/// Sessions audio dont le processus correspond à la source SMTC.
pub(super) fn source_sessions(
    manager: &IAudioSessionManager2,
    source: &str,
) -> windows::core::Result<Vec<IAudioSessionControl>> {
    unsafe {
        let sessions = manager.GetSessionEnumerator()?;
        let mut matching = Vec::new();
        for index in 0..sessions.GetCount()? {
            let control = sessions.GetSession(index)?;
            let pid = control.cast::<IAudioSessionControl2>()?.GetProcessId()?;
            if pid != 0 && process::exe_name(pid).is_some_and(|exe| is_source_matching(source, &exe)) {
                matching.push(control);
            }
        }
        Ok(matching)
    }
}

/// Volume (0.0 à 1.0) le plus fort parmi les sessions, `None` s'il n'y en a aucune.
pub(super) fn max_volume(sessions: &[IAudioSessionControl]) -> windows::core::Result<Option<f32>> {
    let mut volume: Option<f32> = None;
    for session in sessions {
        let level = unsafe { session.cast::<ISimpleAudioVolume>()?.GetMasterVolume()? };
        volume = Some(volume.map_or(level, |v| v.max(level)));
    }
    Ok(volume)
}

/// Applique le volume (0.0 à 1.0) à toutes les sessions ; `WIDGET_CONTEXT` évite que la surveillance le renvoie au front.
pub(super) fn set_volume(sessions: &[IAudioSessionControl], volume: f32) -> windows::core::Result<()> {
    for session in sessions {
        unsafe {
            session
                .cast::<ISimpleAudioVolume>()?
                .SetMasterVolume(volume.clamp(0.0, 1.0), &WIDGET_CONTEXT)?;
        }
    }
    Ok(())
}
