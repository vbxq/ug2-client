use ug2_client::patcher::Patch;
use ug2_client::patcher::patches::experiments::EnableDevExperiments;

#[test]
fn test_dev_experiments() {
    let patch = EnableDevExperiments;
    let input = r#"static displayName="DeveloperExperimentStore";isDeveloper=!1;initialize()"#;
    let result = patch.apply(input);
    assert!(result.contains("isDeveloper=!0"));
    assert!(!result.contains("isDeveloper=!1"));
}
