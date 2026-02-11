# Security & Integrity
**Production Hardening**

Omni Forge enforces strict security boundaries.

## 1. Integrity Verification
- **Hash Checks**: `verify` command recomputes SHA-256 of `mind.bin`.
- **Metadata Validation**: Ensures source provenance and compiler version match.

## 2. Immutability
- **Base Mind**: Loaded as `Arc<MindPack>` (Read-Only). The Runtime *cannot* modify the factory artifact.
- **Overlay**: All writes go to `PersonalMemory`.

## 3. Sandboxing
- **PermissionBoundary**: Restricts file access to `./omniforge_data/` (or configured root).
- **Network**: Disabled by default in Core.

## 4. Tamper Detection
- If `verify_integrity` fails (hash mismatch), the runtime refuses to load (or warns loudly).
- Overlays must match `parent_hash` of the Base Mind. Mismatches trigger a reset or error.
