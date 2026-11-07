use serde::Serialize;

/// Couleurs de Windows (`#rrggbb`), `None` quand elles ne peuvent pas être lues.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemColors {
    pub taskbar: Option<String>,
    pub accent: Option<String>,
}
