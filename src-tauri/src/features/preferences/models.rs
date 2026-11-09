use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Écart (en pixels logiques) en dessous duquel deux positions sont considérées identiques.
const POSITION_EPSILON: f64 = 1.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.id() == id)
    }
}

/// Position du widget sur un écran, en marges (pixels logiques) par rapport
/// au coin bas-droit de cet écran. Indépendant de la résolution et du DPI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorPlacement {
    pub right_margin: f64,
    pub bottom_margin: f64,
}

impl MonitorPlacement {
    pub fn is_close_to(&self, other: &Self) -> bool {
        (self.right_margin - other.right_margin).abs() < POSITION_EPSILON
            && (self.bottom_margin - other.bottom_margin).abs() < POSITION_EPSILON
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub theme: ThemePreference,
    pub last_monitor: Option<String>,
    /// Position personnalisée par écran (clé = `monitor_key`).
    pub monitors: HashMap<String, MonitorPlacement>,
}
