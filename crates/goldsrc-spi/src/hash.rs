//! Centralized deterministic hashing utilities and open-world identity tokens.
//!
//! Provides compile-time `const fn` FNV-1a hashing (64-bit and 32-bit), MurmurHash3,
//! and open-world identifier types (`FeatureToken`, `BackendId`, `ExtensionId`).

pub use crate::identity::murmur3_128;

/// Computes 64-bit FNV-1a hash at compile time.
#[inline(always)]
pub const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    hash
}

/// Computes 64-bit FNV-1a hash of a UTF-8 string at compile time.
#[inline(always)]
pub const fn fnv1a64_str(s: &str) -> u64 {
    fnv1a64(s.as_bytes())
}

/// Computes 32-bit FNV-1a hash at compile time.
#[inline(always)]
pub const fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        i += 1;
    }
    hash
}

/// Computes 32-bit FNV-1a hash of a UTF-8 string at compile time.
#[inline(always)]
pub const fn fnv1a32_str(s: &str) -> u32 {
    fnv1a32(s.as_bytes())
}

use stitch_macros::token;

/// Open-world 64-bit feature token computed at compile-time via FNV-1a.
///
/// Used by game SDKs and plugins to declare, query, and guard game-specific
/// features (e.g. `cstrike:economy`, `cstrike:color_chat`) without core engine
/// knowledge or dynamic allocations.
#[token]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct FeatureToken(pub u64);

impl stitch_core::StitchToken for FeatureToken {
    #[inline(always)]
    fn raw_u64(&self) -> u64 {
        self.0
    }
}

impl FeatureToken {
    /// Computes a feature token from a compile-time string literal.
    #[inline(always)]
    pub const fn from_name(name: &str) -> Self {
        Self(fnv1a64_str(name))
    }

    /// Returns the underlying 64-bit raw numeric value.
    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl From<&str> for FeatureToken {
    #[inline]
    fn from(name: &str) -> Self {
        Self::from_name(name)
    }
}

impl std::fmt::Display for FeatureToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FeatureToken(0x{:016x})", self.0)
    }
}

/// Open-world 64-bit capability token computed at compile-time via FNV-1a.
///
/// Used by ABAC/RBAC authorization, command guards, and plugin permissions
/// without runtime string parsing or dynamic allocations.
#[token]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct CapabilityToken(pub u64);

impl stitch_core::StitchToken for CapabilityToken {
    #[inline(always)]
    fn raw_u64(&self) -> u64 {
        self.0
    }
}

impl CapabilityToken {
    /// Computes a capability token from a compile-time string literal.
    #[inline(always)]
    pub const fn from_name(name: &str) -> Self {
        Self(fnv1a64_str(name))
    }

    /// Returns the underlying 64-bit raw numeric value.
    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl From<&str> for CapabilityToken {
    #[inline]
    fn from(name: &str) -> Self {
        Self::from_name(name)
    }
}

impl std::fmt::Display for CapabilityToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CapabilityToken(0x{:016x})", self.0)
    }
}

/// Open-world typed identifier for engine backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BackendId(pub &'static str);

impl BackendId {
    /// Canon Metamod:Re backend.
    pub const METAMOD: BackendId = BackendId("metamod");
    /// Canon standalone server backend.
    pub const STANDALONE: BackendId = BackendId("standalone");

    /// Creates a custom or mock backend identifier.
    #[inline(always)]
    pub const fn custom(name: &'static str) -> Self {
        Self(name)
    }

    /// Returns the static string representation.
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for BackendId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// Open-world typed identifier for engine extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ExtensionId(pub &'static str);

impl ExtensionId {
    /// ReAPI extension.
    pub const REAPI: ExtensionId = ExtensionId("reapi");
    /// VoiceTranscoder extension.
    pub const VTC: ExtensionId = ExtensionId("vtc");

    /// Creates a custom extension identifier.
    #[inline(always)]
    pub const fn custom(name: &'static str) -> Self {
        Self(name)
    }

    /// Returns the static string representation.
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for ExtensionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a64_const_eval() {
        const TOKEN: FeatureToken = FeatureToken::from_name("cstrike:economy");
        assert_ne!(TOKEN.raw(), 0);
        assert_eq!(TOKEN, FeatureToken::from_name("cstrike:economy"));
        assert_ne!(TOKEN, FeatureToken::from_name("cstrike:color_chat"));
    }

    #[test]
    fn test_fnv1a32_const_eval() {
        const H1: u32 = fnv1a32_str("hello");
        const H2: u32 = fnv1a32_str("hello");
        const H3: u32 = fnv1a32_str("world");
        assert_eq!(H1, H2);
        assert_ne!(H1, H3);
    }

    #[test]
    fn test_open_world_ids() {
        assert_eq!(BackendId::METAMOD.as_str(), "metamod");
        assert_eq!(BackendId::custom("mock").as_str(), "mock");
        assert_eq!(ExtensionId::REAPI.as_str(), "reapi");
        assert_eq!(ExtensionId::custom("amxx_compat").as_str(), "amxx_compat");
    }
}
