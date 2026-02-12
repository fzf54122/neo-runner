use crate::PluginInfo;

pub fn plugin_name() -> &'static str {
    "http"
}

pub fn info() -> PluginInfo {
    PluginInfo {
        name: plugin_name(),
        description: "Execute HTTP request task",
    }
}

pub fn validate(method: Option<&str>, url: Option<&str>) -> Result<(), String> {
    if method.map(str::trim).filter(|v| !v.is_empty()).is_none() {
        return Err("http task requires method".to_string());
    }
    if url.map(str::trim).filter(|v| !v.is_empty()).is_none() {
        return Err("http task requires url".to_string());
    }
    Ok(())
}
