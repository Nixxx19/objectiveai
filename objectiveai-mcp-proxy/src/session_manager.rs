//! Session registry.
//!
//! Maps session ids to [`Session`]s. A proxy session id is the base62
//! encoding of a JSON-serialized `IndexMap<upstream_url, upstream_session_id>`.
//! Because `IndexMap` preserves insertion order and `serde_json` walks
//! the map in that order, the id is fully determined by the (ordered)
//! contents — so re-serializing after a successful all-upstreams resume
//! produces the byte-identical id, while any rotated upstream session
//! produces a different id. Clients can therefore tell at a glance
//! whether anything changed during resume.
//!
//! All per-session dispatch (list, call, read) lives on [`Session`]
//! itself; this file only cares about computing/minting ids, packing
//! connections into a [`Session`], and looking sessions back up.

use std::sync::Arc;
use dashmap::DashMap;
use indexmap::IndexMap;
use objectiveai::mcp::Connection;

use crate::session::Session;

/// Maps a session id to its [`Session`] state.
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: DashMap<String, Arc<Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a session whose id is computed from the upstream
    /// connections in their iteration order. Returns the id.
    ///
    /// The id is `base62(JSON({ url: upstream_session_id, ... }))` with
    /// keys in the order the connections appear in the `Vec`. If a later
    /// `add` is called with the same URLs in the same order resolving to
    /// the same upstream session ids, the returned id is byte-identical
    /// to the previous one — that's the property the orchestrator relies
    /// on to detect "no upstream rotated during resume."
    ///
    /// Connections are keyed inside the [`Session`] by their upstream
    /// `server_info.name`. Duplicate names get `_<index>` suffixes so
    /// downstream tool routing stays unambiguous.
    pub fn add(&self, connections: Vec<Connection>) -> String {
        let id = compute_session_id(&connections);
        let by_name = build_by_name_map(connections);
        self.sessions
            .insert(id.clone(), Arc::new(Session::new(by_name)));
        id
    }

    /// Cheap clone-out of a [`Session`] — never holds a DashMap guard
    /// across the await boundary.
    pub fn get(&self, session_id: &str) -> Option<Arc<Session>> {
        self.sessions.get(session_id).map(|e| e.value().clone())
    }

    /// Remove a session from the registry. Returns `Some(_)` if a session
    /// was present, `None` if the id was unknown.
    ///
    /// Once every `Arc<Session>` to the removed session has dropped, the
    /// session's `IndexMap<String, Connection>` drops, every `Connection`'s
    /// `Drop` fires its upstream's wakeup signal, and each upstream's
    /// listener task wakes to re-check liveness. The listener sees
    /// `Arc::strong_count == 1` (only itself) and exits, which drops the
    /// inner state and closes the upstream HTTP session.
    pub fn remove(&self, session_id: &str) -> Option<Arc<Session>> {
        self.sessions.remove(session_id).map(|(_, session)| session)
    }
}

/// Decode an incoming session id back into the URL→upstream-session-id
/// map it encodes. `None` on any decode failure (bad base62, bad JSON,
/// wrong shape).
pub fn decode_session_id(id: &str) -> Option<IndexMap<String, String>> {
    let bytes = base62_decode_bytes(id)?;
    serde_json::from_slice(&bytes).ok()
}

/// Build an `IndexMap<url, upstream_session_id>` in connection order,
/// JSON-serialize it, and base62-encode the bytes for transport. The
/// ordering is what guarantees byte-stable ids on idempotent resumes.
pub fn compute_session_id(connections: &[Connection]) -> String {
    let mut map: IndexMap<String, String> =
        IndexMap::with_capacity(connections.len());
    for c in connections {
        map.insert(c.url.clone(), c.session_id.clone());
    }
    let json = serde_json::to_vec(&map).expect("IndexMap<String,String> serializes");
    base62_encode_bytes(&json)
}

/// Byte-level base62. The off-the-shelf `base62` crate only encodes
/// `u128`s; we need variable-length input for JSON-encoded session
/// maps. Encoding interprets the bytes as a big-endian unsigned
/// big-integer and prints it in base62 with `0..9 a..z A..Z` digits;
/// leading zero bytes are encoded as a `0` digit each so they survive
/// the round-trip.
fn base62_encode_bytes(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    const ALPHABET: &[u8; 62] =
        b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    // Count leading zeros so we can re-emit them on decode.
    let leading_zeros = bytes.iter().take_while(|b| **b == 0).count();
    // Convert bytes to a vector of base-62 digits (most significant first).
    let mut digits: Vec<u8> = Vec::with_capacity(bytes.len() * 2);
    let mut num: Vec<u32> = bytes[leading_zeros..].iter().map(|b| *b as u32).collect();
    while !num.is_empty() {
        let mut remainder: u32 = 0;
        let mut next: Vec<u32> = Vec::with_capacity(num.len());
        for &b in &num {
            let acc = remainder * 256 + b;
            let q = acc / 62;
            remainder = acc % 62;
            if !(next.is_empty() && q == 0) {
                next.push(q);
            }
        }
        digits.push(remainder as u8);
        num = next;
    }
    let mut out = String::with_capacity(leading_zeros + digits.len());
    for _ in 0..leading_zeros {
        out.push(ALPHABET[0] as char);
    }
    for d in digits.into_iter().rev() {
        out.push(ALPHABET[d as usize] as char);
    }
    out
}

fn base62_decode_bytes(s: &str) -> Option<Vec<u8>> {
    if s.is_empty() {
        return Some(Vec::new());
    }
    fn digit(c: char) -> Option<u32> {
        match c {
            '0'..='9' => Some(c as u32 - '0' as u32),
            'a'..='z' => Some(c as u32 - 'a' as u32 + 10),
            'A'..='Z' => Some(c as u32 - 'A' as u32 + 36),
            _ => None,
        }
    }
    let leading_zeros = s.chars().take_while(|c| *c == '0').count();
    let mut num: Vec<u32> = Vec::with_capacity(s.len());
    for c in s.chars().skip(leading_zeros) {
        num.push(digit(c)?);
    }
    let mut bytes: Vec<u8> = Vec::new();
    while !num.is_empty() {
        let mut remainder: u32 = 0;
        let mut next: Vec<u32> = Vec::with_capacity(num.len());
        for &d in &num {
            let acc = remainder * 62 + d;
            let q = acc / 256;
            remainder = acc % 256;
            if !(next.is_empty() && q == 0) {
                next.push(q);
            }
        }
        bytes.push(remainder as u8);
        num = next;
    }
    let mut out = vec![0u8; leading_zeros];
    out.extend(bytes.into_iter().rev());
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base62_round_trip() {
        for sample in [
            &b""[..],
            &b"a"[..],
            &b"\x00\x01\x02"[..],
            &b"hello world"[..],
            br#"{"http://127.0.0.1:1234":"abc123"}"#,
            &(0..=255u16).map(|b| b as u8).collect::<Vec<_>>()[..],
        ] {
            let encoded = base62_encode_bytes(sample);
            assert!(encoded.bytes().all(|b| (0x21..=0x7E).contains(&b)));
            let decoded = base62_decode_bytes(&encoded).expect("decode");
            assert_eq!(decoded, sample, "round-trip failed for {sample:?}");
        }
    }
}

fn build_by_name_map(
    connections: Vec<Connection>,
) -> IndexMap<String, Connection> {
    // First pass: which names are duplicated? Anything that shows up
    // more than once in the input gets the `_<index>` suffix.
    let mut name_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for c in &connections {
        *name_counts
            .entry(c.initialize_result.server_info.name.clone())
            .or_insert(0) += 1;
    }
    let mut by_name: IndexMap<String, Connection> =
        IndexMap::with_capacity(connections.len());
    for (idx, connection) in connections.into_iter().enumerate() {
        let raw = connection.initialize_result.server_info.name.clone();
        let key = if name_counts.get(&raw).copied().unwrap_or(0) > 1 {
            format!("{raw}_{idx}")
        } else {
            raw
        };
        if by_name.contains_key(&key) {
            tracing::warn!(
                key = %key,
                "two upstreams produce the same prefix after disambiguation; later upstream wins",
            );
        }
        by_name.insert(key, connection);
    }
    by_name
}
