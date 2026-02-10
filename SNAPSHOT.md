# Snapshot Format: The MindPack
**Extension**: `.mindpack` (or `.mindpack.zip`)

A MindPack is a Zip archive containing:

1.  `metadata.json`:
    - `core_hash`: SHA-256 integrity hash.
    - `version`: Engine version.
    - `source`: Origin string.

2.  `manifest.json`:
    - Instructions for runtime instantiation.
    - Capabilities required.

3.  `mind.bin`:
    - `bincode` serialized `CognitionCore`.
    - Hypervector graph, Index memory.

4.  `schema.json`:
    - Encoder configuration.

5.  `limits.json`:
    - Learning policies (max concepts, decay rate).

6.  `integrity.hash`:
    - Separate hash file for quick verification.

**Compression**: Deflated (Lossless).
**Endianness**: Little Endian.

## Security
- `verify` command recalculates hash of `mind.bin`.
- Metadata guarantees provenance.
