use crate::PluginInfo;

pub fn plugin_name() -> &'static str {
    "sql"
}

pub fn info() -> PluginInfo {
    PluginInfo {
        name: plugin_name(),
        description: "Execute SQL batch task",
    }
}

pub fn validate(
    dsn: Option<&str>,
    query: Option<&str>,
    sql_file: Option<&str>,
) -> Result<(), String> {
    if dsn.map(str::trim).filter(|v| !v.is_empty()).is_none() {
        return Err("sql task requires dsn".to_string());
    }
    let has_query = query.map(str::trim).filter(|v| !v.is_empty()).is_some();
    let has_file = sql_file.map(str::trim).filter(|v| !v.is_empty()).is_some();
    if !(has_query || has_file) {
        return Err("sql task requires query or sql_file".to_string());
    }
    Ok(())
}
