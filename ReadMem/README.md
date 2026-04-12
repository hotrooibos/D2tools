# D2R Health Reader

A Rust-based tool that reads Diablo 2 Resurrected's memory to display your character's health and mana stats.

## Features

- Real-time HP and Mana display
- Shows current/max values and percentages
- Automatically detects D2R process
- Single .exe file (no dependencies)

## Usage

1. Run Diablo 2 Resurrected
2. Execute `D2R-HealthReader.exe`
3. The tool will display your HP and MP in real-time

## Output Example

```
==============================================
  Diablo 2 Resurrected - Health Reader
==============================================

Looking for D2R...
(Make sure the game is running!)

Found D2R process!

Press Ctrl+C to exit
==============================================

[Sorceress] HP: 847/1234 (68.6%) | MP: 234/450 (52.0%)
```

## Building from Source

```bash
# Install Rust if needed: https://rustup.rs/
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/d2r-health-reader.exe ./D2R-HealthReader.exe
```

## Notes

- Requires Windows
- Memory addresses may change with game patches
- Read-only access (no injection/modification)
- Use at your own risk (reading process memory may trigger anti-cheat)
