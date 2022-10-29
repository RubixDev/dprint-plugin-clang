use dprint_core::configuration::{
    self, ConfigKeyMap, ConfigKeyValue, GlobalConfiguration, NewLineKind,
    ResolveConfigurationResult,
};

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub new_line_kind: NewLineKind,

    // TODO: language overrides
    #[serde(flatten)]
    pub settings: ConfigKeyMap,
}

impl Configuration {
    pub fn to_cli_flag(&self, text: &str) -> String {
        let mut settings = vec![];
        if configuration::resolve_new_line_kind(text, self.new_line_kind) == "\r\n" {
            settings.push("UseCRLF: true".to_string());
        }
        for (key, value) in &self.settings {
            settings.push(format!(
                "{key}: {value}",
                value = config_key_value_to_string(value),
            ))
        }
        format!("--style={{{}}}", settings.join(", "))
    }
}

fn config_key_value_to_string(value: &ConfigKeyValue) -> String {
    match value {
        ConfigKeyValue::Null => "null".to_string(),
        ConfigKeyValue::String(value) => format!("\"{value}\""),
        ConfigKeyValue::Number(value) => value.to_string(),
        ConfigKeyValue::Bool(value) => value.to_string(),
        ConfigKeyValue::Array(value) => format!(
            "[{}]",
            value
                .iter()
                .map(config_key_value_to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ConfigKeyValue::Object(value) => format!(
            "{{{}}}",
            value
                .iter()
                .map(|(key, value)| format!(
                    "{key}: {value}",
                    value = config_key_value_to_string(value),
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

pub fn resolve_config(
    mut config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
    let mut diagnostics = vec![];
    let mut settings = ConfigKeyMap::new();

    let new_line_kind = configuration::get_value(
        &mut config,
        "newLineKind",
        global_config.new_line_kind.unwrap_or(NewLineKind::LineFeed),
        &mut diagnostics,
    );
    if config.get("ColumnLimit").is_none() {
        if let Some(global) = global_config.line_width {
            settings.insert(
                "ColumnLimit".to_string(),
                ConfigKeyValue::Number(global as i32),
            );
        }
    }
    if config.get("UseTab").is_none() {
        if let Some(true) = global_config.use_tabs {
            settings.insert(
                "UseTab".to_string(),
                ConfigKeyValue::String("Always".to_string()),
            );
        }
    }
    if config.get("IndentWidth").is_none() {
        if let Some(global) = global_config.indent_width {
            settings.insert(
                "IndentWidth".to_string(),
                ConfigKeyValue::Number(global as i32),
            );
        }
    }
    match config.remove("BasedOnStyle") {
        Some(style) => settings.insert("BasedOnStyle".to_string(), style),
        None => settings.insert(
            "BasedOnStyle".to_string(),
            ConfigKeyValue::String("InheritParentConfig".to_string()),
        ),
    };

    settings.extend(config);

    ResolveConfigurationResult {
        diagnostics,
        config: Configuration {
            new_line_kind,
            settings,
        },
    }
}
