use kime_engine_core::{parse_config_from_str, RawConfig};

#[test]
fn check_default_config() {
    assert_eq!(
        serde_yaml::to_string(&RawConfig::default()).unwrap(),
        include_str!("../../../../res/default_config.yaml")
    );
}

const CONFIG_WITH_TAG: &str = "
engine:
  default_category: Latin
  global_hotkeys:
    Hangul:
      behavior: !Toggle
        - Hangul
        - Latin
      result: Consume
  category_hotkeys:
    Hangul:
      ControlR:
        behavior: !Mode Hanja
        result: Consume
  mode_hotkeys:
    Math:
      Enter:
        behavior: Commit
        result: ConsumeIfProcessed
";

const CONFIG_WITH_NO_TAG: &str = "
engine:
  default_category: Latin
  global_hotkeys:
    Hangul:
      behavior:
        Toggle:
        - Hangul
        - Latin
      result: Consume
  category_hotkeys:
    Hangul:
      ControlR:
        behavior:
          Mode: Hanja
        result: Consume
  mode_hotkeys:
    Math:
      Enter:
        behavior: Commit
        result: ConsumeIfProcessed
";

#[test]
fn compat_parse_with_tag() {
    parse_config_from_str(CONFIG_WITH_TAG).expect("with tag");
}

#[test]
fn compat_parse_with_no_tag() {
    parse_config_from_str(CONFIG_WITH_NO_TAG).expect("with no tag");
}
