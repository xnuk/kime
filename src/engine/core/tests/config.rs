#[test]
fn check_default_config() {
    let expect = serde_yaml::to_value(kime_engine_core::RawConfig::default()).unwrap();
    let actual: serde_yaml::Value =
        serde_yaml::from_str(include_str!("../../../../res/default_config.yaml")).unwrap();
    assert_eq!(expect, actual);
}
