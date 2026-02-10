# Snapshot & Distribution Guide
**Sharing Your Creation**

This guide explains how to freeze your Forge into a **MindPack** and distribute it to others.

## 1. Creating a Snapshot
A snapshot freezes the current state of your Forge (including ingested knowledge, vocabulary, and core logic) into an immutable artifact.

```bash
./omniforge snapshot --name MyAI_v1
```
This produces: **MyAI_v1.mindpack** (Raw artifact)

## 2. Compressing for Distribution
To share your MindPack with friends or deploy it to other machines, compress it into a universal format.

```bash
./omniforge compress MyAI_v1.mindpack
```
This produces: **MyAI_v1.mindpack.zip** (Compressed, Lossless)

## 3. Receiver Instructions (For your Friend)
When you send the `.zip` file to someone else, they can run it immediately without modifying the original.

1.  **Download** `MyAI_v1.mindpack.zip`.
2.  **Run**:
    ```bash
    ./omniforge run MyAI_v1.mindpack.zip
    ```
    This will:
    *   Unpack the Mind into memory (Read-Only).
    *   Create a local overlay at `./omniforge_data/overlays/MyAI_v1.local`.
    *   Remember *their* interactions, without changing *your* original file.

## 4. Safety Guarantees
*   **Immutability:** The Base MindPack (`.mindpack`) is never modified by the receiver.
*   **Sovereignty:** All new learning stays on the receiver's device in the `.local` overlay.
*   **Integrity:** The `.mindpack` contains a SHA-256 hash that `omniforge verify` checks.

## 5. Architectural Compliance
*   **No Telemetry:** Omni Forge sends zero data.
*   **No Cloud:** Everything runs locally.
*   **No Auto-Update:** Only you can issue a new `.mindpack` version.
