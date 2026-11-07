use serde::{Serialize, Serializer};

/// Erreur renvoyée par les commandes Tauri : le front la reçoit sous forme de texte.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
