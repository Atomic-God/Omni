# Missing Features Checklist

The following features are stubbed or pending specific hardware integration (Phase 9/10):

## Perception
- [ ] **Real Neural Encoders:** `VisionEncoder` currently returns deterministic hashes. Needs `ort` (ONNX Runtime) or `tch` (LibTorch) implementation.
- [ ] **Audio FFT:** `AudioEncoder` extracts metadata but not spectral features (needs `rustfft` or neural audio model).
- [ ] **Syntax Embedding:** `SyntaxEncoder` does not yet use a real language model (e.g., BERT-tiny).

## Memory
- [ ] **Lazy Loading Optimization:** `LSHIndex` currently holds a copy of vectors in RAM. For >10M items, this needs to store only `ShardLocation` and fetch from disk on query.
- [ ] **Advanced Merge:** Snapshot merging is basic. Conflict resolution for `PersonalDelta` needs refinement.

## Reasoning
- [ ] **Complex Abduction:** Multi-step inference (A->B->C) is currently limited to single-step associations in `AbductiveReasoner`.
- [ ] **Goal Planning:** The `Decide` step uses a simple priority queue. A full planner (A* in VSA space?) is a future upgrade.

## Ingestion
- [ ] **Deep Archive Inspection:** Recursive unpacking is basic. Protection against Zip Bombs is minimal (file count/size checks needed).
- [ ] **OCR:** Image text extraction is not implemented (requires Tesseract/GPU).

## Runtime
- [ ] **Real Actions:** `Act` currently just logs or manipulates the goal queue. Integration with OS (e.g., `fs`, `net`) is needed for a true "Agent".
