# Forge vs Runtime

Strict separation of concerns ensures stability and scalability.

## The Forge (Fabricator)
-   **Role:** The Factory.
-   **Capabilities:** Unlimited memory growth, massive ingestion, schema evolution.
-   **Output:** Immutable Mind Snapshots (`.omf`).
-   **State:** Mutable Master Mind.

## The Runtime (Sovereign)
-   **Role:** The Agent.
-   **Capabilities:** Inference, Local Learning, Personalization.
-   **Memory:**
    -   **Base:** Read-Only (from Snapshot).
    -   **Overlay:** Read-Write (Personal Memory).
-   **Constraints:** Memory budgeted, sandboxed.

## Interaction
1.  Forge ingests world data -> Creates Snapshot v1.0.
2.  User downloads v1.0 -> Runs Runtime.
3.  Runtime learns "My name is Alice" -> Saved to Overlay.
4.  Forge releases v2.0 -> User updates Base.
5.  Runtime v2.0 loads -> "My name is Alice" persists (Overlay migration).
