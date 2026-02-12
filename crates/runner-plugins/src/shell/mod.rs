use crate::PluginInfo;

pub fn plugin_name() -> &'static str {
    "shell"
}

pub fn info() -> PluginInfo {
    PluginInfo {
        name: plugin_name(),
        description: "Execute shell command task",
    }
}

pub fn validate(cmd: Option<&str>) -> Result<(), String> {
    match cmd.map(str::trim) {
        Some(v) if !v.is_empty() => Ok(()),
        _ => Err("shell task requires non-empty cmd".to_string()),
    }
}
