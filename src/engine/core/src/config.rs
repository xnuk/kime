use crate::KeyMap;
use fontdb::{Family, Query};
pub use kime_engine_config::*;
use std::{fs, io};

use serde_yaml::value::Value as YamlValue;

#[derive(Debug)]
pub enum SerdeError {
    SerdeYaml(serde_yaml::Error),
    SerdeJson(serde_json::Error),
}

impl From<serde_yaml::Error> for SerdeError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::SerdeYaml(value)
    }
}

impl From<serde_json::Error> for SerdeError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

impl std::fmt::Display for SerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerdeError::SerdeYaml(x) => x.fmt(f),
            SerdeError::SerdeJson(x) => x.fmt(f),
        }
    }
}
impl std::error::Error for SerdeError {}

fn sanitize(value: YamlValue) -> YamlValue {
    match value {
        YamlValue::Tagged(x) => {
            let key = format!("{}", x.tag).strip_prefix('!');
            let value = sanitize(x.value);
            let mut map = serde_yaml::value::Mapping::with_capacity(1);
            map.insert(YamlValue::String(key), value);
            YamlValue::Mapping(map)
        }
        YamlValue::Sequence(x) => YamlValue::Sequence(x.into_iter().map(sanitize).collect()),
        YamlValue::Mapping(x) => YamlValue::Mapping(
            x.into_iter()
                .map(|(key, value)| (sanitize(key), sanitize(value)))
                .collect(),
        ),
        x => x,
    }
}

pub fn parse_from_yaml_value(value: YamlValue) -> Result<RawConfig, SerdeError> {
    let val = serde_json::to_value(sanitize(value))?;
    Ok(serde_json::from_value(val)?)
}

pub fn parse_config_from_str(x: &str) -> Result<RawConfig, SerdeError> {
    let val: YamlValue = serde_yaml::from_str(x)?;
    Ok(parse_from_yaml_value(val)?)
}

pub fn parse_config_from_reader(x: impl io::Read) -> Result<RawConfig, SerdeError> {
    let val: YamlValue = serde_yaml::from_reader(x)?;
    Ok(parse_from_yaml_value(val)?)
}

/// Preprocessed engine config
pub struct Config {
    pub translation_layer: Option<KeyMap<Key>>,
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub category_hotkeys: EnumMap<InputCategory, Vec<(Key, Hotkey)>>,
    pub mode_hotkeys: EnumMap<InputMode, Vec<(Key, Hotkey)>>,
    pub candidate_font: (Vec<u8>, u32),
    pub xim_preedit_font: (Vec<u8>, u32, f32),
    pub hangul_data: HangulData,
    pub preferred_direct: bool,
    pub latin_data: LatinData,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

impl Config {
    fn new_impl(mut engine: EngineConfig, hangul_data: HangulData) -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let load_font = |name| {
            db.query(&Query {
                families: &[Family::Name(name), Family::Name("D2Coding")],
                ..Default::default()
            })
            .and_then(|id| db.with_face_data(id, |data, index| (data.to_vec(), index)))
            .unwrap_or_default()
        };

        #[cfg(unix)]
        let translation_layer: Option<KeyMap<Key>> = engine
            .translation_layer
            .and_then(|f| {
                xdg::BaseDirectories::with_prefix("kime")
                    .ok()
                    .and_then(|d| d.find_config_file(f))
            })
            .as_ref()
            .and_then(|f| fs::read_to_string(f.as_path()).ok())
            .as_ref()
            .and_then(|content| serde_yaml::from_str(content).ok());

        #[cfg(not(unix))]
        let translation_layer = None;

        Self {
            translation_layer: translation_layer,
            default_category: engine.default_category,
            global_category_state: engine.global_category_state,
            category_hotkeys: enum_map! {
                cat => {
                    if let Some(map) = engine.category_hotkeys.get_mut(&cat) { for (k, v) in engine.global_hotkeys.iter() {
                            map.entry(*k).or_insert(*v);
                        }
                        map.iter().map(|(k, v)| (*k, *v)).collect()
                    } else {
                        engine.global_hotkeys.iter().map(|(k, v)| (*k, *v)).collect()
                    }
                }
            },
            mode_hotkeys: enum_map! {
                mode => {
                    if let Some(map) = engine.mode_hotkeys.get_mut(&mode) {
                        for (k, v) in engine.global_hotkeys.iter() {
                            map.entry(*k).or_insert(*v);
                        }
                        map.iter().map(|(k, v)| (*k, *v)).collect()
                    } else {
                        engine.global_hotkeys.iter().map(|(k, v)| (*k, *v)).collect()
                    }
                }
            },
            xim_preedit_font: {
                let (font, index) = load_font(&engine.xim_preedit_font.0);
                (font, index, engine.xim_preedit_font.1)
            },
            candidate_font: {
                let (font, index) = load_font(&engine.candidate_font);
                (font, index)
            },
            preferred_direct: engine.latin.preferred_direct,
            latin_data: LatinData::new(&engine.latin),
            hangul_data,
        }
    }

    pub fn new(engine: EngineConfig) -> Self {
        let hangul_data = HangulData::new(
            &engine.hangul,
            kime_engine_backend_hangul::builtin_layouts(),
        );

        Self::new_impl(engine, hangul_data)
    }

    #[cfg(unix)]
    pub fn from_engine_config_with_dir(engine: EngineConfig, dir: &xdg::BaseDirectories) -> Self {
        let hangul_data = HangulData::from_config_with_dir(&engine.hangul, dir);
        Self::new_impl(engine, hangul_data)
    }
}

#[cfg(unix)]
pub fn load_engine_config_from_config_dir() -> Option<Config> {
    let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
    let config: RawConfig = dir
        .find_config_file("config.yaml")
        .and_then(|config| parse_config_from_reader(std::fs::File::open(config).ok()?).ok())
        .unwrap_or_default();

    Some(Config::from_engine_config_with_dir(config.engine, &dir))
}

#[cfg(unix)]
pub fn load_other_configs_from_config_dir() -> Option<(DaemonConfig, IndicatorConfig, LogConfig)> {
    let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
    let config: RawConfig = dir
        .find_config_file("config.yaml")
        .and_then(|config| parse_config_from_reader(std::fs::File::open(config).ok()?).ok())
        .unwrap_or_default();

    Some((config.daemon, config.indicator, config.log))
}
