# Safety: MIRI and FFI Guarantees

RLG interfaces with OS kernels via C-FFI for `os_log` (macOS) and `journald` (Linux). This page documents the verification strategy and safety boundaries.

---

## 1. Lock-Free Concurrency

The engine uses `crossbeam::ArrayQueue` instead of `Mutex<T>`. Multiple application threads push events concurrently; a single flusher thread drains them. Memory visibility relies on atomic acquire/release semantics — no locks on the hot path.

## 2. MIRI Verification

Every CI run executes the full test suite under [MIRI](https://github.com/rust-lang/miri), the Rust MIR interpreter:

```bash
MIRIFLAGS="-Zmiri-tree-borrows" cargo miri test
```

MIRI checks for:

- **Pointer provenance violations** — pointers passed to `os_log` or socket calls never escape their valid region.
- **Alignment errors** — stack-allocated `itoa` buffers meet CPU-native alignment requirements.
- **Data races** — no two threads access mutable memory without proper synchronisation.

Tests that spawn OS threads or touch real sockets are `#[cfg_attr(miri, ignore)]` — MIRI cannot emulate kernel syscalls.

## 3. FFI Boundaries

### macOS `os_log`

```rust
// SAFETY: `subsystem` and `category` are valid, null-terminated CStrings.
// Their lifetimes outlive the FFI call.
unsafe {
    let handle = os_log_create(subsystem.as_ptr(), category.as_ptr());
}
```

Every `unsafe` block carries a `// SAFETY:` comment documenting the invariant it relies on.

### Linux `journald`

The Linux sink uses safe Rust (`UnixDatagram`). The binary payload follows the systemd native protocol specification. No `unsafe` is required.

## 4. Stack-Based Formatting

The flusher formats numeric values with `itoa` (integers) and `ryu` (floats) — both write to stack buffers, avoiding heap allocation. The format buffer itself is a reusable `String` that grows once and persists across flush cycles.

This reduces the surface area for OOM conditions under sustained high throughput.

## 5. Summary

| Guarantee | Mechanism |
|-----------|-----------|
| No data races | `crossbeam::ArrayQueue` + atomics |
| No use-after-free in FFI | `CString` lifetime outlives every call |
| No provenance violations | MIRI `-Zmiri-tree-borrows` on every CI run |
| No alignment faults | `itoa`/`ryu` stack buffers verified by MIRI |
| No lock contention | Flusher thread unparked via cached `Thread` handle |
