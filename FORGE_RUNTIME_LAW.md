# The Law of Forge-Runtime Duality
**Version 1.0 (Immutable)**

## 1. The Separation of Powers
The Omni Forge architecture is built upon a strict separation between the **Creator (Forge)** and the **Consumer (Runtime)**.

### 1.1 The Forge (Factory)
*   **Role:** The Forge is the source of truth. It ingests raw reality, distills it into concepts, and fabricates Minds.
*   **Privilege:** The Forge has unlimited memory and compute access (within hardware limits). It can rewrite the Base Mind.
*   **Constraint:** The Forge does not run on the Edge. It resides on the Workstation or Cloud Node.

### 1.2 The Runtime (Sovereign)
*   **Role:** The Runtime is the deployed Mind. It serves the user on their sovereign device (phone, laptop, embedded).
*   **Privilege:** The Runtime is strictly **Local-First**. No data leaves the device.
*   **Constraint:** The Runtime **cannot** modify the Base Mind. It can only write to the **Personal Overlay**.

## 2. The Principle of Immutability
*   The **Base Mind** (the artifact created by the Forge) is **Read-Only** in the Runtime environment.
*   All new learning, memories, and adaptations must be stored in the **Personal Overlay** (`.local` or `.overlay` file).
*   This ensures that the core logic remains uncorrupted, while the user's experience is personalized.

## 3. The Sovereign Data Covenant
*   **No Telemetry:** The Runtime shall never emit data to a central server.
*   **No Auto-Update:** The Runtime shall not accept over-the-air updates that modify the Base Mind without explicit user consent (re-flashing).
*   **Ownership:** The Personal Overlay belongs solely to the user. It is encrypted (future work) and portable.

## 4. The Hardware Truth
*   The System shall respect the hardware it runs on.
*   It shall not attempt to emulate hardware features (like AVX-512) that do not exist, to preserve efficiency.
*   It shall degrade gracefully if resources are low (Low Memory Margin).

---
*Signed,*
*The Architect*
