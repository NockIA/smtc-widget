use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Écart (en pixels logiques) en dessous duquel deux positions sont considérées identiques.
const POSITION_EPSILON: f64 = 1.0;

/// Écart d'échelle en dessous duquel deux tailles sont considérées identiques.
const SCALE_EPSILON: f64 = 0.01;

/// Taille de base du widget (100 %).
pub const DEFAULT_SCALE: f64 = 1.0;

fn default_scale() -> f64 {
    DEFAULT_SCALE
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
    Transparent,
}

impl ThemePreference {
    pub const ALL: [Self; 4] = [Self::System, Self::Light, Self::Dark, Self::Transparent];

    pub fn id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Transparent => "transparent",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.id() == id)
    }
}

/// Position et taille du widget sur un écran : marges (pixels logiques) par rapport
/// au coin bas-droit de cet écran, et échelle (1.0 = taille de base). Indépendant de la résolution et du DPI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorPlacement {
    pub right_margin: f64,
    pub bottom_margin: f64,
    /// Absent des préférences enregistrées avant l'ajout du redimensionnement.
    #[serde(default = "default_scale")]
    pub scale: f64,
}

impl MonitorPlacement {
    pub fn is_close_to(&self, other: &Self) -> bool {
        (self.right_margin - other.right_margin).abs() < POSITION_EPSILON
            && (self.bottom_margin - other.bottom_margin).abs() < POSITION_EPSILON
            && (self.scale - other.scale).abs() < SCALE_EPSILON
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
