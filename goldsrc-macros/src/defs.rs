//! Structural definitions for plugin attributes, commands, and ECS systems.

/// Parsed `#[plugin(...)]` and stacked helper attributes values.
#[derive(Debug, Clone)]
pub struct PluginAttr {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub url: String,
    pub license: String,
    pub bundle: Option<String>,
    pub role: Option<String>,
    pub command_prefix: String,
    pub requires: Vec<String>,
    pub permissions: Vec<String>,
    pub load_time: String,
    pub unload_time: String,
}

/// Information about a registered command definition.
#[derive(Debug, Clone)]
pub struct CommandDefInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
    pub capability: Option<String>,
    pub requires: Vec<String>,
}

/// Constant names for method marker attributes recognized by `#[plugin]`.
pub mod markers {
    pub const ON_LOAD: &str = "on_load";
    pub const ON_UNLOAD: &str = "on_unload";
    pub const ON_FRAME: &str = "on_frame";
    pub const PERMISSIONS: &str = "permissions";
    pub const PERMISSION: &str = "permission";
    pub const EVENT: &str = "event";
    pub const COMMAND: &str = "command";
    pub const MENU_ACTION: &str = "menu_action";
}
