//! Modular engine Hardware Abstraction Layer (HAL) SPI traits and composite Engine interface.

pub mod console;
pub mod cvars;
pub mod entities;
pub mod extensions;
pub mod messages;
pub mod physics;
pub mod precache;
pub mod sound;

pub use console::EngineConsole;
pub use cvars::EngineCvars;
pub use entities::EngineEntities;
pub use extensions::EngineExtensions;
pub use messages::{EngineMessages, MessageBuilder, MessageDest};
pub use physics::{EnginePhysics, TraceResult};
pub use precache::EnginePrecache;
pub use sound::EngineSound;

use stitch_macros::port;

/// Composite engine Hardware Abstraction Layer (HAL) bridge interface.
///
/// Combines modular sub-system traits into a unified driver interface implemented
/// by backends (Metamod, Standalone) and consumed by runtime hosts (`Arc<dyn Engine>`).
#[port]
pub trait EnginePort:
    EnginePrecache
    + EngineMessages
    + EngineEntities
    + EngineCvars
    + EnginePhysics
    + EngineSound
    + EngineConsole
    + EngineExtensions
    + Send
    + Sync
{
}

/// Backward compatibility alias for [`EnginePort`].
pub use EnginePort as Engine;

// Blanket implementation for any type implementing all engine sub-traits.
impl<T> EnginePort for T where
    T: EnginePrecache
        + EngineMessages
        + EngineEntities
        + EngineCvars
        + EnginePhysics
        + EngineSound
        + EngineConsole
        + EngineExtensions
        + Send
        + Sync
{
}
