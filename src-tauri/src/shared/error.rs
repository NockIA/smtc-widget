use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

/// Erreur renvoyée par les commandes Tauri.
///
/// Le détail (message Windows, chemin, etc.) reste dans les logs : le front ne reçoit
/// qu'un code stable (`{ "code": "..." }`) et décide seul de la réaction.
/// Une absence de session ou de piste n'est pas une erreur : elle passe par `Ok(None)` / `Ok(false)`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("SMTC indisponible : {0}")]
    Smtc(windows::core::Error),
    #[error("audio indisponible : {0}")]
    Audio(windows::core::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl AppError {
    /// Code exposé au front.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Smtc(_) => "smtc_unavailable",
            Self::Audio(_) => "audio_unavailable",
            Self::Tauri(_) | Self::Io(_) => "internal",
        }
    }
}

impl Serialize for AppError {
    /// Appelé une seule fois par erreur renvoyée au front : c'est là que le détail est journalisé.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        log::error!("commande a échoué [{}] : {self}", self.code());
        let mut state = serializer.serialize_struct("AppError", 1)?;
        state.serialize_field("code", self.code())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
