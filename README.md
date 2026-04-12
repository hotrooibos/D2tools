# Diablo 2 Resurrected Tools

Collection of utilities for Diablo 2 Resurrected.

## Tools Overview

| Tool | Description |
|------|-------------|
| [save_backup/](save_backup/) | Watch and backup D2R save files automatically |
| [ReadMem/](ReadMem/) | Real-time HP/Mana display via memory reading |
| [MultiInstances/](MultiInstances/) | Launch multiple D2R instances |
| [Lootfilter/](Lootfilter/) | Filter configuration files |

---

## save_backup/

Python tool to watch and backup D2R save files.

### Features

- Watches save folder for changes using `watchdog`
- Creates timestamped ZIP backups (fetched from timeapi.io)
- Manual backup with `b` key (no Enter needed)
- Restore menu to recover from backups
- Configurable via `config.json`
- Automatic cleanup (keeps last N backups)

### Installation

```bash
cd save_backup
pip install -r requirements.txt
```

### Configuration

Edit `config.json` to match your system:

```json
{
    "base_save_folder": "C:/Users/You/Saved Games/Diablo II Resurrected",
    "files_to_save": ["Focus.*", "Mule*", "*.d2i"],
    "output_subfolder": "CharsBak",
    "time_api_url": "https://timeapi.io/api/Time/current/zone?timeZone=Europe/Paris",
    "min_backup_interval": 400,
    "max_backups": 20,
    "debounce_seconds": 10,
    "d2r_exe": "C:/Program Files (x86)/Diablo II Resurrected/D2R.exe"
}
```

### Usage

```bash
python savebackup.py
```

Menu options:
1. Restore from backup
2. Run watcher
3. Start D2R
4. Exit

---

## ReadMem/

Rust memory reader for D2R HP/Mana display. **Windows only.**

### Features

- Real-time HP and Mana display
- Shows current/max values and percentages
- Automatically detects D2R process
- Single executable, no dependencies

### Building from Source

```bash
# Install Rust: https://rustup.rs/
cd ReadMem
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/d2r-health-reader.exe ./D2R-HealthReader.exe
```

### Usage

1. Run Diablo 2 Resurrected
2. Execute `D2R-HealthReader.exe`
3. View HP and Mana in real-time

### Example Output

```
==============================================
  Diablo 2 Resurrected - Health Reader
==============================================

Looking for D2R...

Found D2R process!

Press Ctrl+C to exit
==============================================

[Sorceress] HP: 847/1234 (68.6%) | MP: 234/450 (52.0%)
```

### Notes

- Memory addresses may change with game patches
- Read-only access (no injection/modification)
- May trigger anti-cheat systems

---

## MultiInstances/

Python script for launching multiple D2R instances.

### Requirements

- Windows
- [handle64.exe](https://learn.microsoft.com/sysinternals/downloads.handle) (Sysinternals)
- [AutoHotkey](https://www.autohotkey.com/) (optional, for macros)

### Configuration

Paths are hardcoded in `d2rinstances_manager.py`. Edit the script to match your installation:

```python
d2r_exe = r"C:\Program Files (x86)\Diablo II Resurrected\Diablo II Resurrected Launcher.exe"
d2r_alt_exe = r"C:\Program Files (x86)\Diablo II Resurrected_alt\Diablo II Resurrected Launcher.exe"
```

### Usage

```bash
python d2rinstances_manager.py
```

---

## Lootfilter/

Filter configuration files for D2R mods.

---

## License

MIT License (see [LICENSE](LICENSE))
