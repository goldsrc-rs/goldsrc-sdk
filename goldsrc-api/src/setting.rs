//! Pure, strongly-typed setting domain abstractions (`Setting<T>` and `Settings`).
//!
//! Provides zero-cost configuration leafs, validation bounds, atomic/interior-mutable
//! value storage, change listeners, and aggregate schema introspection.
//!
//! All external formats (TOML, JSON, KDL, Postcard) and engine CVars act strictly
//! as external codecs and adapters over this domain.

use std::fmt;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex, RwLock};

/// Validation constraint on a setting value.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingBounds<T> {
    /// No bounds constraints.
    None,
    /// Value must be within inclusive range `[min, max]`.
    Range(RangeInclusive<T>),
    /// Value must match one of discrete allowed choices.
    Choices(Vec<T>),
}

impl<T: PartialOrd + Clone> SettingBounds<T> {
    /// Validates whether `value` satisfies the bounds.
    #[inline]
    pub fn is_valid(&self, value: &T) -> bool {
        match self {
            Self::None => true,
            Self::Range(range) => range.contains(value),
            Self::Choices(choices) => choices.contains(value),
        }
    }

    /// Clamps `value` to the nearest bound if bounds are an inclusive range.
    #[inline]
    pub fn clamp(&self, value: T) -> T
    where
        T: Ord,
    {
        match self {
            Self::Range(range) => value.clamp(range.start().clone(), range.end().clone()),
            _ => value,
        }
    }
}

/// Metadata describing a single setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingMeta {
    /// Canonical setting identifier (e.g. `regen_amount`).
    pub key: &'static str,
    /// Default string representation.
    pub default_repr: &'static str,
    /// Human-readable description/documentation.
    pub description: &'static str,
    /// Associated category or section name (e.g. `gameplay`, `network`).
    pub section: Option<&'static str>,
}

/// Change observer callback invoked when a setting value is updated.
pub type SettingObserver<T> = Arc<Mutex<Box<dyn Fn(&T, &T) + Send + Sync + 'static>>>;

/// A strongly-typed configuration setting leaf.
///
/// Holds the current value, default fallback, validation bounds, and change observers.
/// Thread-safe via `RwLock` for zero-cost concurrent reading on game frame ticks.
pub struct Setting<T> {
    meta: SettingMeta,
    default_value: T,
    current_value: RwLock<T>,
    bounds: SettingBounds<T>,
    observers: Mutex<Vec<SettingObserver<T>>>,
}

impl<T: Clone + PartialEq + fmt::Debug> fmt::Debug for Setting<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let current = self.get();
        f.debug_struct("Setting")
            .field("key", &self.meta.key)
            .field("current", &current)
            .field("default", &self.default_value)
            .field("description", &self.meta.description)
            .finish()
    }
}

impl<T: Clone + PartialEq> Setting<T> {
    /// Creates a new `Setting` with default value, metadata, and optional bounds.
    pub fn new(meta: SettingMeta, default_value: T, bounds: SettingBounds<T>) -> Self {
        Self {
            meta,
            current_value: RwLock::new(default_value.clone()),
            default_value,
            bounds,
            observers: Mutex::new(Vec::new()),
        }
    }

    /// Returns metadata descriptor for this setting.
    #[inline]
    pub fn meta(&self) -> &SettingMeta {
        &self.meta
    }

    /// Returns the setting key.
    #[inline]
    pub fn key(&self) -> &'static str {
        self.meta.key
    }

    /// Returns the default value.
    #[inline]
    pub fn default_value(&self) -> &T {
        &self.default_value
    }

    /// Reads the current setting value (cloned).
    #[inline]
    pub fn get(&self) -> T {
        let lock = match self.current_value.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        lock.clone()
    }

    /// Sets a new value if it satisfies the validation bounds.
    ///
    /// Returns `Ok(true)` if value was changed, `Ok(false)` if value was already equal,
    /// or `Err(SettingError::OutOfBounds)` if validation failed.
    pub fn set(&self, new_value: T) -> Result<bool, SettingError>
    where
        T: PartialOrd,
    {
        if !self.bounds.is_valid(&new_value) {
            return Err(SettingError::OutOfBounds {
                key: self.meta.key,
                reason: "value outside defined setting bounds".to_string(),
            });
        }

        let mut lock = match self.current_value.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        if *lock == new_value {
            return Ok(false);
        }

        let old_value = lock.clone();
        *lock = new_value.clone();
        drop(lock);

        // Notify observers
        let obs_lock = match self.observers.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        for obs in obs_lock.iter() {
            if let Ok(callback) = obs.lock() {
                callback(&old_value, &new_value);
            }
        }

        Ok(true)
    }

    /// Resets the setting back to its default value.
    pub fn reset_to_default(&self) -> Result<bool, SettingError>
    where
        T: PartialOrd,
    {
        self.set(self.default_value.clone())
    }

    /// Registers a change observer callback.
    pub fn on_change<F>(&self, callback: F)
    where
        F: Fn(&T, &T) + Send + Sync + 'static,
    {
        let mut obs_lock = match self.observers.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        obs_lock.push(Arc::new(Mutex::new(Box::new(callback))));
    }
}

/// Generic error during setting operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingError {
    /// Provided value does not satisfy bounds.
    OutOfBounds {
        /// Setting identifier.
        key: &'static str,
        /// Detail explanation.
        reason: String,
    },
    /// Parse or conversion error from external representation.
    ParseError {
        /// Setting identifier.
        key: &'static str,
        /// Detail explanation.
        reason: String,
    },
}

impl fmt::Display for SettingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { key, reason } => write!(f, "setting '{key}' error: {reason}"),
            Self::ParseError { key, reason } => write!(f, "setting '{key}' parse error: {reason}"),
        }
    }
}

impl std::error::Error for SettingError {}

/// Abstract representation of key-value configuration tree used by codec adapters.
pub type SettingTree = std::collections::BTreeMap<String, String>;

/// Aggregate configuration model trait implemented by structs with `#[derive(Settings)]`.
///
/// Decoupled from all serialization formats (TOML/JSON/KDL/Cvars).
pub trait Settings: Send + Sync {
    /// Schema of all settings contained in this aggregate.
    fn schema() -> Vec<SettingMeta>;

    /// Loads values from an abstract key-value tree.
    fn load_from_tree(&self, tree: &SettingTree) -> Result<(), SettingError>;

    /// Exports current values to an abstract key-value tree.
    fn export_tree(&self) -> SettingTree;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_lifecycle_and_bounds() {
        let meta = SettingMeta {
            key: "regen_hp",
            default_repr: "10.0",
            description: "HP restored per second",
            section: Some("gameplay"),
        };
        let setting = Setting::new(meta, 10.0, SettingBounds::Range(1.0..=50.0));

        assert_eq!(setting.get(), 10.0);
        assert_eq!(setting.key(), "regen_hp");

        // Valid update
        assert!(setting.set(25.0).unwrap());
        assert_eq!(setting.get(), 25.0);

        // Out of bounds update rejected
        let err = setting.set(100.0).unwrap_err();
        assert!(matches!(err, SettingError::OutOfBounds { .. }));
        assert_eq!(setting.get(), 25.0); // unchanged

        // Reset to default
        assert!(setting.reset_to_default().unwrap());
        assert_eq!(setting.get(), 10.0);
    }

    #[test]
    fn test_setting_observer_notification() {
        let meta = SettingMeta {
            key: "enabled",
            default_repr: "true",
            description: "Feature toggle",
            section: None,
        };
        let setting = Setting::new(meta, true, SettingBounds::None);

        let notified = Arc::new(Mutex::new(None));
        let notified_clone = Arc::clone(&notified);

        setting.on_change(move |old, new| {
            let mut lock = notified_clone.lock().unwrap();
            *lock = Some((*old, *new));
        });

        assert!(setting.set(false).unwrap());
        assert_eq!(*notified.lock().unwrap(), Some((true, false)));
    }
}
