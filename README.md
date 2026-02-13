# Phosphor

Desktop GUI for Proxmark3. Scan, clone and manage RFID/NFC cards without touching the command line.

![Windows](https://img.shields.io/badge/Windows-10%2B-blue) ![License](https://img.shields.io/badge/license-GPL--3.0-green) ![Version](https://img.shields.io/badge/version-1.1.0-brightgreen)

## What it does

Phosphor wraps the Proxmark3 client into a visual wizard. You plug in your Proxmark, place a card on the reader, and Phosphor handles the rest: identifying the card type, reading its data, detecting the right blank, and writing the clone. The whole process is point-and-click.

**LF (125 kHz)** cards are cloned in seconds. **HF (13.56 MHz)** cards like MIFARE Classic go through automatic key recovery (autopwn) with real-time progress, then write to a magic card.

## Supported cards

### LF (125 kHz) - 22 types

HID ProxII, EM4100, AWID, IOProx, Indala, FDX-B, HID Corporate 1000, Paradox, Keri, Viking, Visa2000, Noralsy, Presco, Jablotron, NexWatch, PAC/Stanley, SecuraKey, Gallagher, GProxII, Pyramid, NEDAP, T55x7

### HF (13.56 MHz) - 6 types

MIFARE Classic 1K/4K (with autopwn key recovery), MIFARE Ultralight, NTAG, iCLASS/PicoPass, DESFire (detection only, non-cloneable)

### Supported magic blanks

T5577 (LF), Gen1a, Gen2/CUID, Gen3, Gen4 GTU, Gen4 GDM/USCUID (HF)

## Requirements

- **Proxmark3** device (Easy, RDV4, or compatible clone)
- **Windows 10** or later (x64)
- USB cable (data cable, not charge-only)

Proxmark3 firmware v4.20728+ recommended. Phosphor bundles its own PM3 client binary, so you don't need a separate Proxmark3 installation.

## Installation

1. Download `Phosphor_1.1.0_x64-setup.exe` from [Releases](../../releases)
2. Run the installer
3. Plug in your Proxmark3
4. Launch Phosphor

## Features

- **One-click cloning** for LF and HF cards
- **Auto-detection** of card type and frequency
- **MIFARE Classic autopwn** with live progress (dictionary, nested, darkside, hardnested attacks)
- **Magic card detection** identifies Gen1a through Gen4 GDM
- **Blank card data check** warns if the blank already has data written to it
- **Firmware flash** with variant picker (RDV4, RDV4+BT, Generic)
- **T5577 chip detection** and password-protected chip handling
- **Sound effects** and terminal-style UI

## Building from source

```bash
# Prerequisites: Node.js 18+, Rust 1.70+, Proxmark3 client binary

git clone https://github.com/user/phosphor.git
cd phosphor
npm install
npx tauri dev      # development
npx tauri build    # production build
```

The PM3 client binary and its DLLs go in `src-tauri/binaries/` and `src-tauri/pm3-libs/`. See `tauri.conf.json` for the resource mapping.

## Tech stack

Tauri v2, React 19, TypeScript, XState v5, Rust. Dual state machine architecture: Rust backend (WizardMachine) and frontend (XState) stay in sync through Tauri commands.

## License

[GPL-3.0](LICENSE)
