use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Media::Audio::{
    IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume,
    MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::{Interface, PWSTR};

use super::constants::{MIN_EXE_STEM_LEN, PATH_BUFFER_LEN};
use crate::shared::smtc::{resolve_session, smtc_manager, source_id};

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

fn process_exe_name(pid: u32) -> Option<String> {
    unsafe {
        // Processus protégé ou déjà terminé : normal, on passe à la session suivante.
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .inspect_err(|e| log::debug!("OpenProcess({pid}) refusé : {e}"))
            .ok()?;

        let mut buffer = [0u16; PATH_BUFFER_LEN];
        let mut len = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        result
            .inspect_err(|e| log::debug!("QueryFullProcessImageNameW({pid}) échoué : {e}"))
            .ok()?;

        let path = String::from_utf16_lossy(&buffer[..len as usize]);
        path.rsplit('\\').next().map(str::to_owned)
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
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
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
            let Some(exe) = process_exe_name(pid) else {
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
    let manager = smtc_manager()?;
    Ok(resolve_session(&manager, selected).map(|session| source_id(&session)))
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
