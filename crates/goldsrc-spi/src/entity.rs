//! Entity spawning and manipulation provider interface.

/// Abstract entity spawning operations required for entity creation and configuration.
pub trait EntitySpawner: Send + Sync {
    /// Creates a new named entity (e.g. "env_sprite", "info_target").
    /// Returns the newly allocated entity index.
    fn create_named_entity(&self, classname: &str) -> Option<i32>;

    /// Sets an entity's world position.
    fn entity_set_origin(&self, index: i32, pos: [f32; 3]);

    /// Sets an entity's rotation angles.
    fn entity_set_angles(&self, index: i32, angles: [f32; 3]);

    /// Sets an entity key-value string property (`pfnKeyValue`).
    /// Returns `true` if handled.
    fn entity_key_value(&self, index: i32, key: &str, value: &str) -> bool;

    /// Dispatches spawn call on the entity (`pfnSpawn`).
    fn dispatch_spawn(&self, index: i32) -> i32;
}

/// Opaque handle representing the raw GoldSrc engine `edict_t` pointer.
///
/// Prevents accidental sizeof/offset mismatches across the engine boundary
/// while allowing safe pointer transfer between SPI drivers and hosts.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdictHandle(pub *mut core::ffi::c_void);

// SAFETY: Edict handles represent raw engine pointers managed by the engine thread.
unsafe impl Send for EdictHandle {}
unsafe impl Sync for EdictHandle {}

/// Backward compatibility alias for `EdictHandle`.
#[allow(non_camel_case_types)]
pub type edict_t = EdictHandle;
