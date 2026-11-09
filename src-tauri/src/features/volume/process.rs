use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::PWSTR;

const PATH_BUFFER_LEN: usize = 260;

/// Nom du fichier exécutable (ex : "Spotify.exe") du processus `pid`, `None` s'il n'est pas lisible.
pub fn exe_name(pid: u32) -> Option<String> {
    unsafe {
        // Processus protégé ou déjà terminé : normal, on passe à la session suivante.
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .inspect_err(|e| log::debug!("OpenProcess({pid}) refusé : {e}"))
            .ok()?;

        let mut buffer = [0u16; PATH_BUFFER_LEN];
        let mut len = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut len);
        let _ = CloseHandle(handle);
        result
            .inspect_err(|e| log::debug!("QueryFullProcessImageNameW({pid}) a échoué : {e}"))
            .ok()?;

        let path = String::from_utf16_lossy(&buffer[..len as usize]);
        path.rsplit('\\').next().map(str::to_owned)
    }
}
