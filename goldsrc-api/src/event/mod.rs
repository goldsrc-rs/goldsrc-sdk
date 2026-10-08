//! Event subscription, priority ordering, and local guest event dispatching.

pub mod registry;

pub use crate::dag::EventPhase;
pub use registry::{
    Event, EventHandler, EventRegistry, EventSubscriberBuilder, EventSubscription, clear_events,
    dispatch_event, subscribe_event,
};

/// Canonical, strongly typed engine and lifecycle events emitted by the host runtime.
///
/// Replaces stringly-typed AMX Mod X legacy strings (`"round_start"`, `"player_post_think"`)
/// with compiler-verified enumeration variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EngineEvent {
    /// Engine frame tick (called every server frame).
    ServerFrame,
    /// Server map initialization and precache phase completed.
    ServerActivate,
    /// Server map deactivation or level shutdown.
    ServerDeactivate,
    /// Level / map transition.
    MapChange,
    /// Player incoming connection request.
    ClientConnect,
    /// Player disconnected from server.
    ClientDisconnect,
    /// Player entity spawned into the active game world.
    ClientPutInServer,
    /// Player setinfo / userinfo data modified.
    ClientUserInfoChanged,
    /// Player pre-think tick before physics/movement simulation.
    PlayerPreThink,
    /// Player post-think tick after physics/movement simulation.
    PlayerPostThink,
    /// User command processing initiated.
    CmdStart,
    /// User command processing concluded.
    CmdEnd,
    /// Client suicide or console kill command issued.
    ClientKill,
    /// Physical touch between two entities.
    EntityTouch,
    /// Player interactive entity 'use' action.
    EntityUse,
    /// Interactive menu item selected by client.
    MenuSelect,
    /// Virtual TakeDamage hook dispatched for an entity.
    EntityTakeDamage,
    /// Virtual Killed hook dispatched for an entity.
    EntityKilled,
    /// Round start gameplay event.
    RoundStart,
    /// Round conclusion gameplay event.
    RoundEnd,
}

impl EngineEvent {
    /// Returns the canonical event wire identifier string.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ServerFrame => "server_frame",
            Self::ServerActivate => "server_activate",
            Self::ServerDeactivate => "server_deactivate",
            Self::MapChange => "map_change",
            Self::ClientConnect => "client_connect",
            Self::ClientDisconnect => "client_disconnect",
            Self::ClientPutInServer => "client_put_in_server",
            Self::ClientUserInfoChanged => "client_user_info_changed",
            Self::PlayerPreThink => "player_pre_think",
            Self::PlayerPostThink => "player_post_think",
            Self::CmdStart => "cmd_start",
            Self::CmdEnd => "cmd_end",
            Self::ClientKill => "client_kill",
            Self::EntityTouch => "entity_touch",
            Self::EntityUse => "entity_use",
            Self::MenuSelect => "menu_select",
            Self::EntityTakeDamage => "entity_take_damage",
            Self::EntityKilled => "entity_killed",
            Self::RoundStart => "round_start",
            Self::RoundEnd => "round_end",
        }
    }
}

impl std::fmt::Display for EngineEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<EngineEvent> for String {
    fn from(ev: EngineEvent) -> Self {
        ev.as_str().to_string()
    }
}
