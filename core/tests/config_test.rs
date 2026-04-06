#[test]
fn default_config_has_all_agents() {
    let config = relay::Config {
        general: relay::GeneralConfig::default(),
        agents: relay::AgentsConfig::default(),
    };
    assert!(config.general.priority.contains(&"codex".to_string()));
    assert!(config.general.priority.contains(&"claude".to_string()));
    assert!(config.general.priority.contains(&"aider".to_string()));
    assert!(config.general.priority.contains(&"gemini".to_string()));
    assert!(config.general.priority.contains(&"ollama".to_string()));
    assert!(config.general.priority.contains(&"openai".to_string()));
    assert!(config.general.priority.contains(&"copilot".to_string()));
    assert!(config.general.priority.contains(&"opencode".to_string()));
}

#[test]
fn default_config_values() {
    let config = relay::GeneralConfig::default();
    assert_eq!(config.max_context_tokens, 8000);
    assert!(config.auto_handoff);
}

#[test]
fn config_save_and_load() {
    let path = std::env::temp_dir().join("relay_test_config.toml");
    relay::Config::save_default(&path).unwrap();
    assert!(path.exists());

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("priority"));
    assert!(content.contains("codex"));
    assert!(content.contains("max_context_tokens"));

    std::fs::remove_file(&path).ok();
}
