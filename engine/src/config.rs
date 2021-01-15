use crate::{keycode::Key, KeyCode, Layout};
use ahash::AHashSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ComposeConfig {
    pub compose_choseong_ssang: bool,
    pub decompose_choseong_ssang: bool,
    pub compose_jungseong_ssang: bool,
    pub decompose_jungseong_ssang: bool,
    pub compose_jongseong_ssang: bool,
    pub decompose_jongseong_ssang: bool,
}

impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            compose_choseong_ssang: true,
            decompose_choseong_ssang: false,
            compose_jungseong_ssang: false,
            decompose_jungseong_ssang: false,
            compose_jongseong_ssang: false,
            decompose_jongseong_ssang: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RawConfig {
    pub layout: String,
    pub esc_turn_off: bool,
    pub hangul_keys: Vec<String>,
    pub xim_preedit_font: String,
    pub gtk_commit_english: bool,
    pub compose: ComposeConfig,
}

impl Default for RawConfig {
    fn default() -> Self {
        const DEFAULT_HANGUK_KEYS: &[Key] = &[
            Key::normal(KeyCode::AltR),
            Key::normal(KeyCode::Henkan),
            Key::normal(KeyCode::Hangul),
            Key::super_(KeyCode::Space),
        ];

        Self {
            layout: "dubeolsik".to_string(),
            esc_turn_off: true,
            hangul_keys: DEFAULT_HANGUK_KEYS
                .iter()
                .map(ToString::to_string)
                .collect(),
            xim_preedit_font: "D2Coding".to_string(),
            gtk_commit_english: true,
            compose: ComposeConfig::default(),
        }
    }
}

pub struct Config {
    pub(crate) layout: Layout,
    pub(crate) esc_turn_off: bool,
    pub(crate) hangul_keys: AHashSet<Key>,
    pub(crate) compose: ComposeConfig,
    pub xim_preedit_font: String,
    pub gtk_commit_english: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(Layout::default(), RawConfig::default())
    }
}

impl Config {
    pub fn new(layout: Layout, raw: RawConfig) -> Self {
        Self {
            layout,
            esc_turn_off: raw.esc_turn_off,
            compose: raw.compose,
            hangul_keys: raw
                .hangul_keys
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect(),
            xim_preedit_font: raw.xim_preedit_font,
            gtk_commit_english: raw.gtk_commit_english,
        }
    }

    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;

        let config = match dir.find_config_file("config.yaml") {
            Some(config) => config,
            None => {
                let path = dir.place_config_file("config.yaml").ok()?;
                std::fs::write(&path, serde_yaml::to_string(&RawConfig::default()).ok()?).ok()?;
                path
            }
        };

        let config: RawConfig =
            serde_yaml::from_reader(std::fs::File::open(config).ok()?).unwrap_or_default();

        let layout = dir
            .list_data_files("layouts")
            .into_iter()
            .find_map(|layout| {
                if layout.file_stem()?.to_str()? == config.layout {
                    Some(Layout::from_items(
                        serde_yaml::from_reader(std::fs::File::open(layout).ok()?).ok()?,
                    ))
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Some(Self::new(layout, config))
    }
}
