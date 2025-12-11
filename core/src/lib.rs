//! # GBEmu Core
//! 
//! High-accuracy Game Boy (DMG) and Game Boy Color (CGB) emulator core.
//! 
//! ## Architecture
//! 
//! The emulator is built around cycle-accurate components:
//! - **CPU**: Sharp LR35902 (Z80-like) with all instructions and timings
//! - **MMU**: Memory Management Unit handling all memory maps
//! - **PPU**: Pixel Processing Unit with DMG/CGB modes
//! - **APU**: Audio Processing Unit with 4 channels (6 for CGB)
//! - **Timer**: DIV, TIMA, TMA, TAC registers
//! - **Cartridge**: MBC1, MBC2, MBC3 (with RTC), MBC5 support
//! - **Joypad**: Button input handling

#![allow(clippy::new_without_default)]

pub mod cpu;
pub mod mmu;
pub mod ppu;
pub mod apu;
pub mod cartridge;
pub mod timer;
pub mod joypad;
pub mod serial;

#[cfg(feature = "wasm")]
mod wasm;

use cpu::Cpu;
use mmu::Mmu;
use ppu::Ppu;
use apu::Apu;
use timer::Timer;
use joypad::Joypad;
use cartridge::Cartridge;
use serial::Serial;

use serde::{Serialize, Deserialize};

/// Game Boy model type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GbModel {
    /// Original Game Boy (DMG)
    Dmg,
    /// Game Boy Pocket
    Pocket,
    /// Game Boy Color
    Cgb,
    /// Game Boy Color in DMG compatibility mode
    CgbDmg,
}

impl Default for GbModel {
    fn default() -> Self {
        Self::Dmg
    }
}

/// Main emulator state
pub struct GameBoy {
    pub cpu: Cpu,
    pub mmu: Mmu,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timer: Timer,
    pub joypad: Joypad,
    pub serial: Serial,
    pub model: GbModel,
    
    /// Cycles executed this frame
    cycles_this_frame: u32,
    
    /// Total cycles executed
    total_cycles: u64,
    
    /// Frame counter
    frame_count: u64,
}

/// Cycles per frame at ~59.7 FPS
/// CPU runs at 4.194304 MHz, frame rate is 59.7275 Hz
pub const CYCLES_PER_FRAME: u32 = 70224;

/// CPU clock speed in Hz
pub const CPU_CLOCK_HZ: u32 = 4_194_304;

impl GameBoy {
    /// Create a new Game Boy instance with a ROM
    pub fn new(rom_data: &[u8]) -> Result<Self, String> {
        let cartridge = Cartridge::from_rom(rom_data)?;
        let model = if cartridge.is_cgb() {
            GbModel::Cgb
        } else {
            GbModel::Dmg
        };
        
        let mut gb = Self {
            cpu: Cpu::new(),
            mmu: Mmu::new(cartridge, model),
            ppu: Ppu::new(model),
            apu: Apu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            serial: Serial::new(),
            model,
            cycles_this_frame: 0,
            total_cycles: 0,
            frame_count: 0,
        };
        
        // Initialize CPU registers based on model
        gb.cpu.init_for_model(model);
        
        Ok(gb)
    }
    
    /// Reset the emulator
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.init_for_model(self.model);
        self.mmu.reset();
        self.ppu.reset();
        self.apu.reset();
        self.timer.reset();
        self.joypad.reset();
        self.serial.reset();
        self.cycles_this_frame = 0;
        self.total_cycles = 0;
        self.frame_count = 0;
    }
    
    /// Run a single CPU step and synchronize all components
    pub fn step(&mut self) -> u32 {
        // Execute one CPU instruction
        let cycles = self.cpu.step(&mut self.mmu);
        
        // Synchronize all components
        self.sync_components(cycles);
        
        cycles
    }
    
    /// Synchronize all components with CPU cycles
    fn sync_components(&mut self, cycles: u32) {
        // Update timer
        let timer_interrupt = self.timer.step(cycles);
        if timer_interrupt {
            self.mmu.request_interrupt(0x04); // Timer interrupt
        }
        
        // Update OAM DMA (one byte per M-cycle = 4 T-cycles)
        for _ in 0..(cycles / 4).max(1) {
            self.mmu.step_dma();
        }
        
        // Update PPU
        let ppu_result = self.ppu.step(cycles, &mut self.mmu);
        if ppu_result.vblank_interrupt {
            self.mmu.request_interrupt(0x01); // VBlank
        }
        if ppu_result.stat_interrupt {
            self.mmu.request_interrupt(0x02); // STAT
        }
        
        // Process audio register writes
        for (addr, value) in self.mmu.take_audio_writes() {
            self.apu.write_register(addr, value);
        }
        
        // Update APU
        self.apu.step(cycles);
        
        // Update serial
        let serial_interrupt = self.serial.step(cycles);
        if serial_interrupt {
            self.mmu.request_interrupt(0x08); // Serial
        }
        
        // Update joypad (check for interrupt)
        if self.joypad.check_interrupt() {
            self.mmu.request_interrupt(0x10); // Joypad
        }
        
        self.cycles_this_frame += cycles;
        self.total_cycles += cycles as u64;
    }
    
    /// Run until the next frame is complete
    /// Returns the framebuffer
    pub fn run_frame(&mut self) -> &[u8] {
        self.cycles_this_frame = 0;
        
        while self.cycles_this_frame < CYCLES_PER_FRAME {
            self.step();
        }
        
        self.frame_count += 1;
        self.ppu.framebuffer()
    }
    
    /// Run for a specific number of cycles
    pub fn run_cycles(&mut self, target_cycles: u32) {
        let mut cycles_run = 0;
        while cycles_run < target_cycles {
            cycles_run += self.step();
        }
    }
    
    /// Press a button
    pub fn press_button(&mut self, button: Button) {
        self.joypad.press(button);
        self.mmu.update_joypad(&self.joypad);
    }
    
    /// Release a button
    pub fn release_button(&mut self, button: Button) {
        self.joypad.release(button);
        self.mmu.update_joypad(&self.joypad);
    }
    
    /// Get the current framebuffer (RGBA8888, 160x144)
    pub fn framebuffer(&self) -> &[u8] {
        self.ppu.framebuffer()
    }
    
    /// Get audio samples
    pub fn audio_buffer(&self) -> &[f32] {
        self.apu.output_buffer()
    }
    
    /// Clear audio buffer after reading
    pub fn clear_audio_buffer(&mut self) {
        self.apu.clear_buffer();
    }
    
    /// Save SRAM (battery-backed save)
    pub fn save_sram(&self) -> Option<Vec<u8>> {
        self.mmu.cartridge().save_ram()
    }
    
    /// Load SRAM
    pub fn load_sram(&mut self, data: &[u8]) -> Result<(), String> {
        self.mmu.cartridge_mut().load_ram(data)
    }
    
    /// Create a save state
    pub fn save_state(&self) -> Vec<u8> {
        let state = SaveState {
            cpu: self.cpu.state(),
            mmu: self.mmu.state(),
            ppu: self.ppu.state(),
            apu: self.apu.state(),
            timer: self.timer.state(),
            joypad: self.joypad.state(),
            model: self.model,
            cycles_this_frame: self.cycles_this_frame,
            total_cycles: self.total_cycles,
            frame_count: self.frame_count,
        };
        
        serde_json::to_vec(&state).unwrap_or_default()
    }
    
    /// Load a save state
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), String> {
        let state: SaveState = serde_json::from_slice(data)
            .map_err(|e| format!("Failed to parse save state: {}", e))?;
        
        self.cpu.load_state(state.cpu);
        self.mmu.load_state(state.mmu)?;
        self.ppu.load_state(state.ppu);
        self.apu.load_state(state.apu);
        self.timer.load_state(state.timer);
        self.joypad.load_state(state.joypad);
        self.model = state.model;
        self.cycles_this_frame = state.cycles_this_frame;
        self.total_cycles = state.total_cycles;
        self.frame_count = state.frame_count;
        
        Ok(())
    }
    
    /// Get the game title from the cartridge
    pub fn game_title(&self) -> &str {
        self.mmu.cartridge().title()
    }
    
    /// Check if the game is a CGB game
    pub fn is_cgb_game(&self) -> bool {
        self.mmu.cartridge().is_cgb()
    }
    
    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
    
    /// Get total cycles executed
    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }
}

/// Serializable save state
#[derive(serde::Serialize, serde::Deserialize)]
struct SaveState {
    cpu: cpu::CpuState,
    mmu: mmu::MmuState,
    ppu: ppu::PpuState,
    apu: apu::ApuState,
    timer: timer::TimerState,
    joypad: joypad::JoypadState,
    model: GbModel,
    cycles_this_frame: u32,
    total_cycles: u64,
    frame_count: u64,
}

// Re-export public types
pub use joypad::Button;
pub use ppu::{SCREEN_WIDTH, SCREEN_HEIGHT};
pub use apu::SAMPLE_RATE;

#[cfg(feature = "wasm")]
pub use wasm::*;