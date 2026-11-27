// traceparent.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! W3C Trace Context `traceparent` parsing
//! (<https://www.w3.org/TR/trace-context/#traceparent-header>).

/// The trace id and parent span id of a valid `traceparent` value:
/// `version-traceid-parentid-flags`, lowercase hex, version not `ff`,
/// ids not all zeros. Version `00` has exactly four parts; a later
/// version may carry more, which are ignored.
pub(super) fn parse(value: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = value.trim().split('-').collect();
    let [version, trace_id, parent_id, flags, rest @ ..] =
        parts.as_slice()
    else {
        return None;
    };
    let known_shape = *version != "00" || rest.is_empty();
    let valid = known_shape
        && is_version(version)
        && is_id(trace_id, 32)
        && is_id(parent_id, 16)
        && is_hex(flags, 2);
    valid.then(|| ((*trace_id).to_string(), (*parent_id).to_string()))
}

fn is_version(s: &str) -> bool {
    is_hex(s, 2) && s != "ff"
}

fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len
        && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_id(s: &str, len: usize) -> bool {
    is_hex(s, len) && s.bytes().any(|b| b != b'0')
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn a_later_version_may_carry_more_parts() {
        let v = "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-extra";
        assert!(parse(v).is_some());
        let v00 = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-extra";
        assert!(parse(v00).is_none());
    }
}
