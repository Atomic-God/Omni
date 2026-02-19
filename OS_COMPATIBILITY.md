# OS Compatibility

Omni Forge aims for "Write Once, Run Anywhere".

## Supported Targets
-   **Linux:** x86_64, aarch64 (primary dev target).
-   **macOS:** aarch64 (Apple Silicon), x86_64.
-   **Windows:** x86_64 (MSVC).

## Abstraction Layer
-   **Paths:** Uses `directories` crate for XDG/standard paths.
-   **Hardware:** `hte` crate abstracts CPUID and memory stats.
-   **Concurrency:** `std::thread` and `tokio` (optional) handle platform threading.

## Constraints
-   Avoid OS-specific syscalls in core crates.
-   Use `std::path::PathBuf` universally.
