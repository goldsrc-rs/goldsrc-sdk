//! Unified collection and iteration facade for active server players (slots 1..=32).

use crate::client::{ClientExt, Player, Team};
use crate::entity::EntityExt;
use crate::spec::{Alive, Dead, Refined};

/// Unified facade for querying and iterating active server players.
///
/// Replaces archaic AMX Mod X raw slot loops (`for slot in 1..=32`) with
/// zero-allocation, typed iterators.
pub struct Players;

impl Players {
    /// Iterates over all valid connected players on the server (slots 1..=32).
    ///
    /// Automatically filters out disconnected or unallocated client slots.
    #[inline]
    pub fn all() -> impl Iterator<Item = Player> {
        crate::client::PlayerSlot::all().filter_map(|slot| slot.resolve())
    }

    /// Iterates over all currently living players (`is_alive() == true`).
    #[inline]
    pub fn alive() -> impl Iterator<Item = Refined<'static, Player, Alive>> {
        Self::all().filter_map(|p| {
            if p.is_alive() {
                // SAFETY: Verified p.is_alive() is true immediately above.
                Some(unsafe { Refined::new_unchecked(p) })
            } else {
                None
            }
        })
    }

    /// Iterates over all dead players or spectators (`is_alive() == false`).
    #[inline]
    pub fn dead() -> impl Iterator<Item = Refined<'static, Player, Dead>> {
        Self::all().filter_map(|p| {
            if !p.is_alive() {
                // SAFETY: Verified !p.is_alive() is true immediately above.
                Some(unsafe { Refined::new_unchecked(p) })
            } else {
                None
            }
        })
    }

    /// Iterates over all players belonging to the specified team.
    #[inline]
    pub fn team(team: Team) -> impl Iterator<Item = Player> {
        Self::all().filter(move |p| p.team() == team)
    }

    /// Iterates over all human players (excluding fake client bots and HLTV proxies).
    #[inline]
    pub fn humans() -> impl Iterator<Item = Player> {
        Self::all().filter(|p| !p.is_bot() && !p.is_hltv())
    }

    /// Finds a player by slot number (`1`..`32` or `#1`..`#32`), SteamID, or name.
    ///
    /// Precedence order:
    /// 1. Exact numeric slot index.
    /// 2. Exact SteamID / AuthID match.
    /// 3. Exact case-insensitive player name match.
    /// 4. Case-insensitive player name prefix match.
    pub fn find(query: &str) -> Option<Player> {
        let q = query.trim();
        if q.is_empty() {
            return None;
        }

        // 1. Direct slot index check (e.g. "5" or "#5")
        let slot_str = q.strip_prefix('#').unwrap_or(q);
        if let Ok(slot_idx) = slot_str.parse::<i32>()
            && let Some(p) = crate::client::PlayerSlot::new(slot_idx).resolve()
        {
            return Some(p);
        }

        // 2. Exact SteamID or exact name
        let target_steam = crate::client::SteamId::parse(q);
        let mut prefix_match = None;
        for p in Self::all() {
            if let Some(target_sid) = target_steam
                && p.steam_id() == Some(target_sid)
            {
                return Some(p);
            }
            let auth = p.auth_id();
            if !auth.is_empty() && auth.eq_ignore_ascii_case(q) {
                return Some(p);
            }
            if let Some(name) = p.name() {
                if name.eq_ignore_ascii_case(q) {
                    return Some(p);
                }
                if name
                    .to_ascii_lowercase()
                    .starts_with(&q.to_ascii_lowercase())
                    && prefix_match.is_none()
                {
                    prefix_match = Some(p);
                }
            }
        }

        prefix_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_players_find_by_slot() {
        // Querying non-connected slot returns None cleanly
        assert_eq!(Players::find("#99"), None);
        assert_eq!(Players::find(""), None);
    }
}
