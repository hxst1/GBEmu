# GBEmu - Game Boy Emulator

A high-accuracy Game Boy (DMG) and Game Boy Color (CGB) emulator written in Rust, compiled to WebAssembly, with a modern Next.js 15 frontend.

## Features

### Emulation
- ✅ Complete CPU (LR35902) with all instructions and timing
- ✅ PPU with accurate scanline rendering (DMG + CGB modes)
- ✅ APU with all 4 audio channels
- ✅ Timer with DIV, TIMA, TMA, TAC
- ✅ Memory Bank Controllers: MBC0, MBC1, MBC2, MBC3 (with RTC), MBC5
- ✅ Battery-backed SRAM saves
- ✅ Save states

### Frontend
- ✅ Modern Next.js 15 with React Server Components
- ✅ TypeScript 5.7
- ✅ Tailwind CSS 4
- ✅ Touch controls for mobile
- ✅ Keyboard support
- ✅ Audio via Web Audio API
- ✅ PWA support (installable, works offline)
- ✅ IndexedDB for persistent saves
- ✅ iOS Safari compatible

## Project Structure

```
gameboy-emulator/
├── core/                    # Rust emulator core
│   ├── src/
│   │   ├── lib.rs          # Main emulator
│   │   ├── cpu/            # CPU implementation
│   │   ├── mmu/            # Memory management
│   │   ├── ppu/            # Graphics
│   │   ├── apu/            # Audio
│   │   ├── cartridge/      # ROM/MBC handling
│   │   ├── timer/          # Timer
│   │   ├── joypad/         # Input
│   │   ├── serial/         # Link cable (stub)
│   │   └── wasm.rs         # WASM bindings
│   └── Cargo.toml
│
└── web/                     # Next.js frontend
    ├── app/
    │   ├── layout.tsx
    │   ├── page.tsx
    │   └── globals.css
    ├── components/
    │   ├── EmulatorContext.tsx
    │   ├── Screen.tsx
    │   ├── Controls.tsx
    │   ├── RomLoader.tsx
    │   ├── SettingsPanel.tsx
    │   └── SaveManager.tsx
    ├── hooks/
    │   ├── useAudio.ts
    │   └── useSaveData.ts
    ├── lib/wasm/            # WASM output (generated)
    ├── public/
    │   ├── manifest.json
    │   └── sw.js
    └── package.json
```

## Building

### Prerequisites

- Rust 1.78+ with `wasm32-unknown-unknown` target
- wasm-pack
- Node.js 20+
- pnpm (recommended) or npm

### Build WASM Core

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build the WASM module
cd core
wasm-pack build --target web --out-dir ../web/lib/wasm
```

### Build Frontend

```bash
cd web

# Install dependencies
pnpm install

# Development
pnpm dev

# Production build
pnpm build
pnpm start
```

## Usage

1. Open the app in your browser
2. Click "Load ROM" or drag & drop a .gb/.gbc file
3. Play!

### Controls

| Action | Keyboard | Touch |
|--------|----------|-------|
| D-Pad | Arrow Keys / WASD | D-Pad buttons |
| A | Z / K | A button |
| B | X / J | B button |
| Start | Enter | Start button |
| Select | Backspace | Select button |
| Pause | P / Escape | Settings menu |
| Save State | F5 | Settings menu |
| Load State | F8 | Settings menu |

## Compatibility

### Tested Games
- Pokémon Red/Blue/Yellow
- Pokémon Gold/Silver/Crystal
- The Legend of Zelda: Link's Awakening
- Tetris
- Super Mario Land 1/2
- Kirby's Dream Land
- And many more!

### Browser Support
- Chrome/Edge 90+
- Firefox 90+
- Safari 15.4+
- iOS Safari 15.4+

## Technical Details

### CPU
The LR35902 CPU is fully implemented with:
- All 256 base instructions
- All 256 CB-prefixed instructions
- Accurate cycle timing
- Interrupt handling (VBlank, STAT, Timer, Serial, Joypad)
- HALT and STOP modes
- HALT bug emulation

### Memory Bank Controllers
| MBC | ROM Size | RAM Size | Features |
|-----|----------|----------|----------|
| MBC0 | 32KB | 8KB | No banking |
| MBC1 | 2MB | 32KB | ROM/RAM banking |
| MBC2 | 256KB | 512 nibbles | Built-in RAM |
| MBC3 | 2MB | 32KB | RTC support |
| MBC5 | 8MB | 128KB | Large ROM support |

### Audio
All 4 audio channels are implemented:
- Channel 1: Square wave with sweep
- Channel 2: Square wave
- Channel 3: Wave output (custom waveform)
- Channel 4: Noise (LFSR)

Audio is output at 44.1kHz via the Web Audio API.

### Graphics
- 160×144 resolution
- DMG: 4 shades of green
- CGB: 32,768 colors (RGB555)
- OAM with 40 sprites (10 per scanline)
- Background, Window, and Sprite layers

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [Pan Docs](https://gbdev.io/pandocs/) - Comprehensive Game Boy documentation
- [GBEDG](https://hacktix.github.io/GBEDG/) - Game Boy Emulator Development Guide
- [Blargg's test ROMs](https://github.com/retrio/gb-test-roms) - CPU instruction tests
