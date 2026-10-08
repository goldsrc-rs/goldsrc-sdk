//! Engine extension query capability trait.

/// Trait providing engine extension inspection and discovery capabilities.
pub trait EngineExtensions: Send + Sync {
    /// Returns `true` if the specified extension is registered, active, and satisfies
    /// the optional semantic version constraint.
    fn is_extension_available(&self, name: &str, version_req: Option<&str>) -> bool {
        let _ = (name, version_req);
        false
    }

    /// Returns the semantic version of the extension if available.
    fn get_extension_version(&self, name: &str) -> Option<String> {
        let _ = name;
        None
    }

    /// Returns `true` if the specified 64-bit feature token is registered and active.
    fn has_feature(&self, token: u64) -> bool {
        let _ = token;
        false
    }

    /// Returns a list of all registered extensions: `(name, version, is_active)`.
    fn list_extensions(&self) -> Vec<(&'static str, String, bool)> {
        Vec::new()
    }
}
