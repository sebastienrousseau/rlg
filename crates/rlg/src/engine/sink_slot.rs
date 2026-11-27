// sink_slot.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The sink chosen in code, held until the flusher starts.

use crate::sink::PlatformSink;
use std::sync::Mutex;

/// A sink waiting for the flusher, and whether the flusher has
/// already chosen one.
#[derive(Debug)]
pub(super) struct SinkSlot {
    sink: Option<PlatformSink>,
    taken: bool,
}

impl SinkSlot {
    const fn open() -> Self {
        Self {
            sink: None,
            taken: false,
        }
    }

    /// Holds `sink` for the flusher, or hands it back once the
    /// flusher has chosen.
    pub(super) fn set(
        &mut self,
        sink: PlatformSink,
    ) -> Result<(), PlatformSink> {
        if self.taken {
            return Err(sink);
        }
        self.sink = Some(sink);
        Ok(())
    }

    /// The flusher's one call: the sink set in code, if any. Closes
    /// the slot. (Miri runs no flusher.)
    #[cfg(any(not(miri), test))]
    pub(super) const fn take(&mut self) -> Option<PlatformSink> {
        self.taken = true;
        self.sink.take()
    }
}

/// The process-wide slot. Locked only by `set_sink` and once by the
/// flusher at start-up, never on the ingest path.
pub(super) static SINK_SLOT: Mutex<SinkSlot> =
    Mutex::new(SinkSlot::open());

/// Locks the slot, recovering from a poisoned lock (the slot holds no
/// invariant a panic could break).
pub(super) fn with_slot<T>(f: impl FnOnce(&mut SinkSlot) -> T) -> T {
    let mut slot = SINK_SLOT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut slot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sink_set_before_the_flusher_is_taken() {
        let mut slot = SinkSlot::open();
        assert!(slot.set(PlatformSink::Stdout).is_ok());
        assert!(matches!(slot.take(), Some(PlatformSink::Stdout)));
        assert!(slot.taken);
    }

    #[test]
    fn a_sink_set_after_the_flusher_is_refused() {
        let mut slot = SinkSlot::open();
        assert!(slot.take().is_none());
        assert!(matches!(
            slot.set(PlatformSink::Stdout),
            Err(PlatformSink::Stdout)
        ));
        assert!(slot.take().is_none());
    }
}
