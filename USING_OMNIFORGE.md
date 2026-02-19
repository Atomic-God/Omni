# Using Omni Forge (Runtime Guide)
**Version 8.9**

## 1. Portable Directory Mode
Omni Forge is designed to be **Zero-Install** and **Portable**.
All your data (MindPacks, Overlays, Master Forge) lives in:

    ./omniforge_data/

You can move the `omniforge` binary and `omniforge_data` folder anywhere (USB stick, DropBox, etc.) and it will work without configuration.

## 2. Command Reference

### Initialize Your Forge
This creates the core structure and the Master Mind.
```bash
./omniforge init
```

### Check Status
See where your data lives and system health.
```bash
./omniforge status
./omniforge doctor
```

### Ingest Data (Feed the Mind)
Teach your Forge about the world. It accepts text, code, HTML, and documents.
```bash
./omniforge ingest ./my_knowledge_folder
```

### Run a Mind
Launch a sovereign instance. This creates a local memory overlay (`.local`) that remembers your interactions.
```bash
./omniforge run my_mind.mindpack.zip
```
Inside the runtime shell:
*   `/learn <text>`: Teach the mind something new (remembered in overlay).
*   `/save`: Save your session explicitly.
*   `/goals`: List active goals.
*   `exit`: Save and quit.

### Verify Integrity
Check if a MindPack has been tampered with.
```bash
./omniforge verify my_mind.mindpack.zip
```
This ensures the core logic is exactly as the creator intended.
