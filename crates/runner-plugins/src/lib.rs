pub mod http;
pub mod shell;
pub mod sql;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn builtin_plugins() -> Vec<PluginInfo> {
    vec![shell::info(), http::info(), sql::info()]
}

pub fn is_builtin(task_type: &str) -> bool {
    builtin_plugins().iter().any(|p| p.name == task_type)
}
