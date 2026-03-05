# Safety: MIRI and Native FFI Guarantees

In the pursuit of brutalist performance, `rlg` interfaces directly with operating system kernels via C-FFI. This page explains the rigorous safety boundaries and verification processes that ensure `rlg` remains "Rust-Safe" even at the binary edge.

## 1. The Zero-Race Guarantee
`rlg` uses a lock-free architecture. Traditional `Mutex<T>` structures are susceptible to priority inversion and deadlocks. By using `crossbeam-queue::ArrayQueue`, we ensure:
- **Single-Producer, Single-Consumer (at the flusher):** Log events are ingested by multiple application threads but are strictly formatted and emitted by a single background OS thread.
- **Atomic Pointer Swapping:** Memory visibility is managed through atomic acquire/release semantics, ensuring that log payloads are never read before they are fully initialized.

## 2. MIRI Verification
We don't just "hope" our `unsafe` code is correct. Every release of `rlg` is mathematically validated using **MIRI**, the Rust MIR interpreter.

We run tests with:
```bash
MIRIFLAGS="-Zmiri-tree-borrows" cargo miri test
```

### What MIRI Checks:
- **Pointer Provenance:** Ensures that pointers passed to macOS `os_log` or Linux sockets never "leak" into unauthorized memory regions.
- **Alignment:** Validates that stack-allocated `itoa` buffers are correctly aligned for CPU-native integer formatting.
- **Data Races:** Interprets the code through a strict execution model to prove that no two threads are accessing mutable memory simultaneously without proper atomic synchronization.

## 3. The FFI Boundary: `os_log` and `journald`
When we cross from Rust to C (the OS), we apply **IBM-Standard Enterprise Rigor**:

### macOS `os_log` Safety:
```rust
// SAFETY: The pointers passed to `os_log_create` and `_os_log_impl` are derived from
// valid, null-terminated `CString`s. The `buf` pointer is valid for `size` bytes.
unsafe {
    let log_handle = os_log_create(subsystem.as_ptr(), category.as_ptr());
    // ... emission logic ...
}
```
We ensure that the lifetime of the `CString` always outlives the FFI call, preventing use-after-free vulnerabilities.

### Linux `journald` Safety:
Our Linux sink uses `UnixDatagram`. While this is safe Rust, the way we construct the binary payload follows the **Systemd Native Protocol** specification exactly, ensuring that systemd never rejects a malformed packet which could lead to observability "blind spots."

## 4. Stack vs. Heap: Minimizing the Surface Area
By prioritizing stack-based formatting (using `itoa` and `ryu`), we drastically reduce the surface area for memory errors. Heap-based logging is the number one cause of "Out of Memory" (OOM) crashes in high-throughput microservices. `rlg`'s strategy of reusable buffers and stack-locality makes it uniquely resilient to memory pressure.

---

## Technical Summary
`rlg` treats `unsafe` as a precision tool, not a convenience. By combining strict **Diátaxis Reference** documentation with **MIRI-validated** binaries, we provide enterprise users with the performance of C and the safety of Rust.
