# Failure Recovery

Industrial systems must handle crashes gracefully.

## Persistence Strategy
-   **Atomic Writes:** Overlays are saved to `.tmp` then renamed to prevent corruption.
-   **Checkpoints:** Learning engine creates periodic checkpoints.

## Error Handling
-   **No Panics:** Core logic uses `Result<T, E>`.
-   **Graceful Degradation:** If an overlay is corrupted, the system falls back to the immutable base (safe mode).

## Diagnosis
-   **Audit Logs:** All major actions are logged.
-   **Integrity Check:** `omniforge verify` scans artifacts for bit rot.
