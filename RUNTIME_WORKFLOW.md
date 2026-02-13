# Runtime Workflow

## 1. Initialization
```bash
omniforge init --path ./my_mind
```
Creates a secure, portable data directory structure.

## 2. Ingestion (Fabrication)
```bash
omniforge ingest --input ./docs/manuals.pdf
```
Parses content, generates embeddings (via NeuralMapper), and builds a semantic graph in memory.

## 3. Training (Consolidation)
```bash
omniforge train --data ./corpus --epochs 5
```
Trains the neural perception layer on raw text to align embeddings with the VSA core.

## 4. Runtime Execution (OODA)
```bash
omniforge run --snapshot my_mind.snap
```
Starts the autonomous loop:
1.  **Observe:** Read input/sensors.
2.  **Orient:** Update context, calculate surprise.
3.  **Decide:** Prioritize goals (Intent Resolver).
4.  **Act:** Execute commands or internal thought updates.

## 5. Persistence
System automatically consolidates memory (decay/promotion) and saves snapshots on exit or command.
