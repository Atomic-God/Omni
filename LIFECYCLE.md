# OmniForge Lifecycle

## 1. Forge Mode (Training)
- **Role:** Modifies base weights and `ForgeMind` memory.
- **Access:** Read/Write to `MemoryManager`.
- **Output:** New `.snap` artifacts.

## 2. Runtime Mode (Inference)
- **Role:** Loads Base Snapshot (ReadOnly).
- **Updates:** Writes to `PersonalDelta` (Ephemeral or Session-based).
- **OODA:** Active Inference loop with Energy Budget.

## 3. GPU Transition
- **Export:** Use `omniforge export-gpu` to dump Parquet/JSON.
- **Train:** Run PyTorch notebook on Kaggle.
- **Import:** Use `omniforge import-gpu` to load updated weights.
