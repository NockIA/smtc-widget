use crate::features::preferences::ThemePreference;

const AUTOSTART_ID: &str = "autostart";
const QUIT_ID: &str = "quit";
const SOURCE_PREFIX: &str = "source:";
const MONITOR_PREFIX: &str = "monitor:";
const THEME_PREFIX: &str = "theme:";

/// Entrée du menu contextuel ; `id`/`parse` font le lien avec l'identifiant porté par l'élément de menu.
#[derive(Debug, PartialEq, Eq)]
pub enum MenuAction {
    Source(String),
    Monitor(String),
    Theme(ThemePreference),
    Autostart,
    Quit,
}

impl MenuAction {
    pub fn id(&self) -> String {
        match self {
            Self::Source(id) => format!("{SOURCE_PREFIX}{id}"),
            Self::Monitor(key) => format!("{MONITOR_PREFIX}{key}"),
            Self::Theme(theme) => format!("{THEME_PREFIX}{}", theme.id()),
            Self::Autostart => AUTOSTART_ID.to_owned(),
            Self::Quit => QUIT_ID.to_owned(),
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            AUTOSTART_ID => Some(Self::Autostart),
            QUIT_ID => Some(Self::Quit),
            _ => {
                if let Some(source) = id.strip_prefix(SOURCE_PREFIX) {
                    Some(Self::Source(source.to_owned()))
                } else if let Some(key) = id.strip_prefix(MONITOR_PREFIX) {
                    Some(Self::Monitor(key.to_owned()))
                } else {
                    id.strip_prefix(THEME_PREFIX)
                        .and_then(ThemePreference::from_id)
                        .map(Self::Theme)
                }
            }
        }
    }
}
