//! Dedicated server host abstractions and console writer.

/// Server console output writer.
pub struct ServerConsole;

impl ServerConsole {
    /// Prints raw text directly to the dedicated server host console.
    #[inline]
    pub fn print(&self, msg: impl AsRef<str>) {
        let text = msg.as_ref();
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_print_console(0, text);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                && let Some(hook) = *lock
            {
                hook(0, crate::client::PrintTarget::Console, text);
            } else {
                print!("{text}");
            }
        }
    }

    /// Prints a line directly to the dedicated server host console with trailing newline.
    #[inline]
    pub fn println(&self, msg: impl AsRef<str>) {
        let mut s = msg.as_ref().to_string();
        if !s.ends_with('\n') {
            s.push('\n');
        }
        self.print(&s);
    }
}

/// Unified host server facade.
pub struct Server;

impl Server {
    /// Returns the dedicated server console output writer.
    #[inline(always)]
    pub const fn console() -> ServerConsole {
        ServerConsole
    }

    /// Broadcasts a chat message to all connected players on the server.
    #[inline]
    pub fn broadcast_chat(msg: impl AsRef<str>) {
        let text = msg.as_ref();
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_print_chat(-1, text);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = crate::client::player::NATIVE_PRINT_HOOK.read()
                && let Some(hook) = *lock
            {
                hook(0, crate::client::PrintTarget::Chat, text);
            }
        }
    }

    /// Executes a console command on the server engine (`pfnServerCommand`).
    #[inline(always)]
    pub fn command(cmd: impl AsRef<str>) {
        crate::engine::server_command(cmd.as_ref());
    }

    /// Queues a `.cfg` file execution on the server engine (`exec <filename>`).
    #[inline(always)]
    pub fn config_exec(file: impl AsRef<str>) {
        let _ = crate::engine::config_exec(file.as_ref());
    }

    /// Queries if a given 64-bit feature token is supported and active in the host engine.
    #[inline]
    pub fn has_feature(token: crate::FeatureToken) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_has_feature(token.raw())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(lock) = FEATURE_QUERY_HOOK.read()
                && let Some(hook) = *lock
            {
                hook(token.raw())
            } else {
                false
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type FeatureQueryHook = fn(u64) -> bool;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static FEATURE_QUERY_HOOK: std::sync::RwLock<Option<FeatureQueryHook>> =
    std::sync::RwLock::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub fn set_feature_query_hook(hook: FeatureQueryHook) {
    if let Ok(mut lock) = FEATURE_QUERY_HOOK.write() {
        *lock = Some(hook);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_console_write() {
        Server::console().print("test console output");
    }

    #[test]
    fn test_server_has_feature_fallback() {
        assert!(!Server::has_feature(crate::FeatureToken::from_name(
            "cstrike:economy"
        )));
    }
}
