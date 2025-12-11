//! # Memory Management Unit
//! 
//! Handles all memory mapping and I/O for the Game Boy.
//! 
//! ## Memory Map
//! 
//! - 0x0000-0x3FFF: ROM Bank 0 (16KB)
//! - 0x4000-0x7FFF: ROM Bank 1-N (switchable, 16KB)
//! - 0x8000-0x9FFF: VRAM (8KB, banked on CGB)
//! - 0xA000-0xBFFF: External RAM (cartridge)
//! - 0xC000-0xCFFF: WRAM Bank 0 (4KB)
//! - 0xD000-0xDFFF: WRAM Bank 1-7 (4KB, switchable on CGB)
//! - 0xE000-0xFDFF: Echo RAM (mirror of C000-DDFF)
//! - 0xFE00-0xFE9F: OAM (Sprite Attribute Table)
//! - 0xFEA0-0xFEFF: Unusable
//! - 0xFF00-0xFF7F: I/O Registers
//! - 0xFF80-0xFFFE: High RAM (HRAM)
//! - 0xFFFF: Interrupt Enable Register

use crate::cartridge::Cartridge;
use crate::joypad::Joypad;
use crate::GbModel;
use serde::{Serialize, Deserialize};

/// VRAM size per bank (8KB)
const VRAM_SIZE: usize = 0x2000;

/// WRAM size per bank (4KB)
const WRAM_BANK_SIZE: usize = 0x1000;

/// OAM size (160 bytes)
const OAM_SIZE: usize = 0xA0;

/// HRAM size (127 bytes)
const HRAM_SIZE: usize = 0x7F;

/// I/O registers size
const IO_SIZE: usize = 0x80;

/// MMU state for serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct MmuState {
    pub vram: Vec<u8>,
    pub wram: Vec<u8>,
    pub oam: Vec<u8>,
    pub hram: Vec<u8>,
    pub io: Vec<u8>,
    pub ie: u8,
    pub vram_bank: u8,
    pub wram_bank: u8,
    pub dma_active: bool,
    pub dma_byte: u8,
    pub dma_source: u16,
    pub hdma_active: bool,
    pub hdma_source: u16,
    pub hdma_dest: u16,
    pub hdma_length: u8,
    pub hdma_hblank: bool,
}

/// Memory Management Unit
pub struct Mmu {
    /// Cartridge
    cartridge: Cartridge,
    
    /// Video RAM (8KB per bank, 2 banks on CGB)
    vram: Vec<u8>,
    
    /// Work RAM (4KB per bank, 8 banks on CGB)
    wram: Vec<u8>,
    
    /// Object Attribute Memory
    oam: [u8; OAM_SIZE],
    
    /// High RAM
    hram: [u8; HRAM_SIZE],
    
    /// I/O Registers
    io: [u8; IO_SIZE],
    
    /// Interrupt Enable register (0xFFFF)
    ie: u8,
    
    /// Game Boy model
    model: GbModel,
    
    /// Current VRAM bank (CGB only, 0 or 1)
    vram_bank: u8,
    
    /// Current WRAM bank (CGB only, 1-7)
    wram_bank: u8,
    
    /// OAM DMA is active
    dma_active: bool,
    
    /// Current DMA byte being transferred
    dma_byte: u8,
    
    /// DMA source address
    dma_source: u16,
    
    /// HDMA is active (CGB only)
    hdma_active: bool,
    
    /// HDMA source address
    hdma_source: u16,
    
    /// HDMA destination address
    hdma_dest: u16,
    
    /// HDMA remaining length
    hdma_length: u8,
    
    /// HDMA mode (true = HBlank, false = General)
    hdma_hblank: bool,
    
    /// Button state (raw state of all 8 buttons, bit=0 means pressed)
    button_state: u8,
    
    /// Pending audio register writes (addr, value)
    audio_writes: Vec<(u16, u8)>,
}

impl Mmu {
    /// Create a new MMU
    pub fn new(cartridge: Cartridge, model: GbModel) -> Self {
        let is_cgb = matches!(model, GbModel::Cgb | GbModel::CgbDmg);
        
        let vram_banks = if is_cgb { 2 } else { 1 };
        let wram_banks = if is_cgb { 8 } else { 2 };
        
        let mut mmu = Self {
            cartridge,
            vram: vec![0; VRAM_SIZE * vram_banks],
            wram: vec![0; WRAM_BANK_SIZE * wram_banks],
            oam: [0; OAM_SIZE],
            hram: [0; HRAM_SIZE],
            io: [0; IO_SIZE],
            ie: 0,
            model,
            vram_bank: 0,
            wram_bank: 1,
            dma_active: false,
            dma_byte: 0,
            dma_source: 0,
            hdma_active: false,
            hdma_source: 0,
            hdma_dest: 0,
            hdma_length: 0,
            hdma_hblank: false,
            button_state: 0xFF,
            audio_writes: Vec::with_capacity(16),
        };
        
        // Initialize I/O registers to post-boot values
        mmu.init_io_registers();
        
        mmu
    }
    
    /// Initialize I/O registers to post-boot ROM values
    fn init_io_registers(&mut self) {
        // These are the values after the boot ROM completes
        self.io[0x00] = 0xCF; // JOYP
        self.io[0x01] = 0x00; // SB
        self.io[0x02] = 0x7E; // SC
        self.io[0x04] = 0xAB; // DIV
        self.io[0x05] = 0x00; // TIMA
        self.io[0x06] = 0x00; // TMA
        self.io[0x07] = 0xF8; // TAC
        self.io[0x0F] = 0xE1; // IF
        
        // Audio registers
        self.io[0x10] = 0x80; // NR10
        self.io[0x11] = 0xBF; // NR11
        self.io[0x12] = 0xF3; // NR12
        self.io[0x13] = 0xFF; // NR13
        self.io[0x14] = 0xBF; // NR14
        self.io[0x16] = 0x3F; // NR21
        self.io[0x17] = 0x00; // NR22
        self.io[0x18] = 0xFF; // NR23
        self.io[0x19] = 0xBF; // NR24
        self.io[0x1A] = 0x7F; // NR30
        self.io[0x1B] = 0xFF; // NR31
        self.io[0x1C] = 0x9F; // NR32
        self.io[0x1D] = 0xFF; // NR33
        self.io[0x1E] = 0xBF; // NR34
        self.io[0x20] = 0xFF; // NR41
        self.io[0x21] = 0x00; // NR42
        self.io[0x22] = 0x00; // NR43
        self.io[0x23] = 0xBF; // NR44
        self.io[0x24] = 0x77; // NR50
        self.io[0x25] = 0xF3; // NR51
        self.io[0x26] = 0xF1; // NR52
        
        // LCD registers
        self.io[0x40] = 0x91; // LCDC
        self.io[0x41] = 0x85; // STAT
        self.io[0x42] = 0x00; // SCY
        self.io[0x43] = 0x00; // SCX
        self.io[0x44] = 0x00; // LY
        self.io[0x45] = 0x00; // LYC
        self.io[0x47] = 0xFC; // BGP
        self.io[0x48] = 0xFF; // OBP0
        self.io[0x49] = 0xFF; // OBP1
        self.io[0x4A] = 0x00; // WY
        self.io[0x4B] = 0x00; // WX
        
        // CGB-specific
        if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
            self.io[0x4D] = 0xFF; // KEY1 (speed switch)
            self.io[0x4F] = 0xFF; // VBK (VRAM bank)
            self.io[0x70] = 0xFF; // SVBK (WRAM bank)
        }
    }
    
    /// Reset MMU state
    pub fn reset(&mut self) {
        self.vram.fill(0);
        self.wram.fill(0);
        self.oam.fill(0);
        self.hram.fill(0);
        self.io.fill(0);
        self.ie = 0;
        self.vram_bank = 0;
        self.wram_bank = 1;
        self.dma_active = false;
        self.dma_byte = 0;
        self.dma_source = 0;
        self.hdma_active = false;
        self.hdma_source = 0;
        self.hdma_dest = 0;
        self.hdma_length = 0;
        self.hdma_hblank = false;
        self.button_state = 0xFF;
        self.audio_writes.clear();
        
        self.init_io_registers();
    }
    
    /// Read a byte from memory
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 0
            0x0000..=0x3FFF => self.cartridge.read_rom(addr),
            
            // ROM Bank N
            0x4000..=0x7FFF => self.cartridge.read_rom(addr),
            
            // VRAM
            0x8000..=0x9FFF => {
                let offset = (addr - 0x8000) as usize;
                let bank_offset = self.vram_bank as usize * VRAM_SIZE;
                self.vram.get(bank_offset + offset).copied().unwrap_or(0xFF)
            }
            
            // External RAM
            0xA000..=0xBFFF => self.cartridge.read_ram(addr),
            
            // WRAM Bank 0
            0xC000..=0xCFFF => {
                let offset = (addr - 0xC000) as usize;
                self.wram.get(offset).copied().unwrap_or(0xFF)
            }
            
            // WRAM Bank N
            0xD000..=0xDFFF => {
                let offset = (addr - 0xD000) as usize;
                let bank = self.wram_bank.max(1) as usize;
                let bank_offset = bank * WRAM_BANK_SIZE;
                self.wram.get(bank_offset + offset).copied().unwrap_or(0xFF)
            }
            
            // Echo RAM (mirror of C000-DDFF)
            0xE000..=0xFDFF => self.read_byte(addr - 0x2000),
            
            // OAM
            0xFE00..=0xFE9F => {
                // During DMA, OAM is inaccessible
                if self.dma_active {
                    0xFF
                } else {
                    self.oam[(addr - 0xFE00) as usize]
                }
            }
            
            // Unusable
            0xFEA0..=0xFEFF => 0xFF,
            
            // I/O Registers
            0xFF00..=0xFF7F => self.read_io(addr),
            
            // HRAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            
            // Interrupt Enable
            0xFFFF => self.ie,
        }
    }
    
    /// Write a byte to memory
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // ROM (writes go to MBC)
            0x0000..=0x7FFF => self.cartridge.write_rom(addr, value),
            
            // VRAM
            0x8000..=0x9FFF => {
                let offset = (addr - 0x8000) as usize;
                let bank_offset = self.vram_bank as usize * VRAM_SIZE;
                if let Some(byte) = self.vram.get_mut(bank_offset + offset) {
                    *byte = value;
                }
            }
            
            // External RAM
            0xA000..=0xBFFF => self.cartridge.write_ram(addr, value),
            
            // WRAM Bank 0
            0xC000..=0xCFFF => {
                let offset = (addr - 0xC000) as usize;
                if let Some(byte) = self.wram.get_mut(offset) {
                    *byte = value;
                }
            }
            
            // WRAM Bank N
            0xD000..=0xDFFF => {
                let offset = (addr - 0xD000) as usize;
                let bank = self.wram_bank.max(1) as usize;
                let bank_offset = bank * WRAM_BANK_SIZE;
                if let Some(byte) = self.wram.get_mut(bank_offset + offset) {
                    *byte = value;
                }
            }
            
            // Echo RAM
            0xE000..=0xFDFF => self.write_byte(addr - 0x2000, value),
            
            // OAM
            0xFE00..=0xFE9F => {
                if !self.dma_active {
                    self.oam[(addr - 0xFE00) as usize] = value;
                }
            }
            
            // Unusable
            0xFEA0..=0xFEFF => {}
            
            // I/O Registers
            0xFF00..=0xFF7F => self.write_io(addr, value),
            
            // HRAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
            
            // Interrupt Enable
            0xFFFF => self.ie = value,
        }
    }
    
    /// Read from I/O register
    fn read_io(&self, addr: u16) -> u8 {
        let reg = (addr & 0x7F) as usize;
        
        match addr {
            // Joypad - calculate based on selection and button state
            0xFF00 => {
                let select = self.io[0x00];
                let mut result = select | 0xC0; // Bits 6-7 always 1
                
                // Select action buttons (bit 5 = 0)
                if select & 0x20 == 0 {
                    result &= 0xF0 | ((self.button_state >> 4) & 0x0F);
                }
                
                // Select d-pad (bit 4 = 0)
                if select & 0x10 == 0 {
                    result &= 0xF0 | (self.button_state & 0x0F);
                }
                
                result
            }
            
            // Serial transfer data
            0xFF01 => self.io[0x01],
            
            // Serial transfer control
            0xFF02 => self.io[0x02] | 0x7E,
            
            // DIV (upper bits of internal timer)
            0xFF04 => self.io[0x04],
            
            // TIMA
            0xFF05 => self.io[0x05],
            
            // TMA
            0xFF06 => self.io[0x06],
            
            // TAC
            0xFF07 => self.io[0x07] | 0xF8,
            
            // IF (Interrupt Flag)
            0xFF0F => self.io[0x0F] | 0xE0,
            
            // Audio registers
            0xFF10..=0xFF26 => self.io[reg],
            
            // Wave pattern RAM
            0xFF30..=0xFF3F => self.io[reg],
            
            // LCD Control
            0xFF40 => self.io[0x40],
            
            // STAT
            0xFF41 => self.io[0x41] | 0x80,
            
            // SCY
            0xFF42 => self.io[0x42],
            
            // SCX
            0xFF43 => self.io[0x43],
            
            // LY (current scanline)
            0xFF44 => self.io[0x44],
            
            // LYC (LY compare)
            0xFF45 => self.io[0x45],
            
            // DMA
            0xFF46 => self.io[0x46],
            
            // BGP
            0xFF47 => self.io[0x47],
            
            // OBP0
            0xFF48 => self.io[0x48],
            
            // OBP1
            0xFF49 => self.io[0x49],
            
            // WY
            0xFF4A => self.io[0x4A],
            
            // WX
            0xFF4B => self.io[0x4B],
            
            // CGB: KEY1 (speed switch)
            0xFF4D => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x4D] | 0x7E
                } else {
                    0xFF
                }
            }
            
            // CGB: VBK (VRAM bank)
            0xFF4F => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.vram_bank | 0xFE
                } else {
                    0xFF
                }
            }
            
            // CGB: HDMA registers
            0xFF51..=0xFF55 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    match addr {
                        0xFF55 => {
                            if self.hdma_active {
                                self.hdma_length & 0x7F
                            } else {
                                0xFF
                            }
                        }
                        _ => self.io[reg],
                    }
                } else {
                    0xFF
                }
            }
            
            // CGB: Background palette index
            0xFF68 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x68]
                } else {
                    0xFF
                }
            }
            
            // CGB: Background palette data
            0xFF69 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x69]
                } else {
                    0xFF
                }
            }
            
            // CGB: Object palette index
            0xFF6A => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x6A]
                } else {
                    0xFF
                }
            }
            
            // CGB: Object palette data
            0xFF6B => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x6B]
                } else {
                    0xFF
                }
            }
            
            // CGB: SVBK (WRAM bank)
            0xFF70 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.wram_bank | 0xF8
                } else {
                    0xFF
                }
            }
            
            // Undefined I/O
            _ => 0xFF,
        }
    }
    
    /// Write to I/O register
    fn write_io(&mut self, addr: u16, value: u8) {
        let reg = (addr & 0x7F) as usize;
        
        match addr {
            // Joypad
            0xFF00 => {
                // Only bits 4-5 are writable (select lines)
                self.io[0x00] = (self.io[0x00] & 0xCF) | (value & 0x30);
                // Update joypad state based on selection
            }
            
            // Serial
            0xFF01 => self.io[0x01] = value,
            0xFF02 => self.io[0x02] = value,
            
            // DIV - writing any value resets it to 0
            0xFF04 => self.io[0x04] = 0,
            
            // Timer registers
            0xFF05 => self.io[0x05] = value, // TIMA
            0xFF06 => self.io[0x06] = value, // TMA
            0xFF07 => self.io[0x07] = value & 0x07, // TAC
            
            // IF
            0xFF0F => self.io[0x0F] = value & 0x1F,
            
            // Audio registers - store in io AND queue for APU
            0xFF10..=0xFF26 => {
                self.io[reg] = value;
                self.audio_writes.push((addr, value));
            }
            
            // Wave pattern RAM - store in io AND queue for APU
            0xFF30..=0xFF3F => {
                self.io[reg] = value;
                self.audio_writes.push((addr, value));
            }
            
            // LCDC
            0xFF40 => self.io[0x40] = value,
            
            // STAT
            0xFF41 => {
                // Bits 0-2 are read-only (mode and coincidence)
                self.io[0x41] = (self.io[0x41] & 0x07) | (value & 0xF8);
            }
            
            // SCY
            0xFF42 => self.io[0x42] = value,
            
            // SCX
            0xFF43 => self.io[0x43] = value,
            
            // LY is read-only (writing resets it on some models)
            0xFF44 => {}
            
            // LYC
            0xFF45 => self.io[0x45] = value,
            
            // DMA transfer
            0xFF46 => {
                self.io[0x46] = value;
                self.start_dma(value);
            }
            
            // BGP
            0xFF47 => self.io[0x47] = value,
            
            // OBP0
            0xFF48 => self.io[0x48] = value,
            
            // OBP1
            0xFF49 => self.io[0x49] = value,
            
            // WY
            0xFF4A => self.io[0x4A] = value,
            
            // WX
            0xFF4B => self.io[0x4B] = value,
            
            // CGB: KEY1
            0xFF4D => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x4D] = (self.io[0x4D] & 0x80) | (value & 0x01);
                }
            }
            
            // CGB: VBK
            0xFF4F => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.vram_bank = value & 0x01;
                }
            }
            
            // CGB: HDMA source high
            0xFF51 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.hdma_source = (self.hdma_source & 0x00FF) | ((value as u16) << 8);
                }
            }
            
            // CGB: HDMA source low
            0xFF52 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.hdma_source = (self.hdma_source & 0xFF00) | ((value & 0xF0) as u16);
                }
            }
            
            // CGB: HDMA dest high
            0xFF53 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.hdma_dest = (self.hdma_dest & 0x00FF) | (((value & 0x1F) as u16) << 8);
                }
            }
            
            // CGB: HDMA dest low
            0xFF54 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.hdma_dest = (self.hdma_dest & 0xFF00) | ((value & 0xF0) as u16);
                }
            }
            
            // CGB: HDMA control
            0xFF55 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.start_hdma(value);
                }
            }
            
            // CGB: BGPI
            0xFF68 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x68] = value;
                }
            }
            
            // CGB: BGPD
            0xFF69 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x69] = value;
                    // Auto-increment if bit 7 is set
                    if self.io[0x68] & 0x80 != 0 {
                        self.io[0x68] = (self.io[0x68] & 0xC0) | ((self.io[0x68] + 1) & 0x3F);
                    }
                }
            }
            
            // CGB: OBPI
            0xFF6A => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x6A] = value;
                }
            }
            
            // CGB: OBPD
            0xFF6B => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.io[0x6B] = value;
                    // Auto-increment if bit 7 is set
                    if self.io[0x6A] & 0x80 != 0 {
                        self.io[0x6A] = (self.io[0x6A] & 0xC0) | ((self.io[0x6A] + 1) & 0x3F);
                    }
                }
            }
            
            // CGB: SVBK
            0xFF70 => {
                if matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
                    self.wram_bank = (value & 0x07).max(1);
                }
            }
            
            _ => {}
        }
    }
    
    /// Start OAM DMA transfer
    fn start_dma(&mut self, value: u8) {
        self.dma_active = true;
        self.dma_byte = 0;
        self.dma_source = (value as u16) << 8;
    }
    
    /// Step DMA transfer (call each M-cycle)
    pub fn step_dma(&mut self) {
        if !self.dma_active {
            return;
        }
        
        let src = self.dma_source + self.dma_byte as u16;
        let value = self.read_byte(src);
        self.oam[self.dma_byte as usize] = value;
        
        self.dma_byte += 1;
        if self.dma_byte >= 160 {
            self.dma_active = false;
        }
    }
    
    /// Start HDMA transfer (CGB only)
    fn start_hdma(&mut self, value: u8) {
        if value & 0x80 == 0 {
            // General purpose DMA
            self.hdma_length = value & 0x7F;
            self.hdma_hblank = false;
            self.hdma_active = true;
            
            // Transfer immediately
            self.run_general_hdma();
        } else {
            // HBlank DMA
            self.hdma_length = value & 0x7F;
            self.hdma_hblank = true;
            self.hdma_active = true;
        }
    }
    
    /// Run general purpose HDMA (all at once)
    fn run_general_hdma(&mut self) {
        let blocks = self.hdma_length as u16 + 1;
        
        for _ in 0..blocks {
            for i in 0..16u16 {
                let src = self.hdma_source + i;
                let dst = 0x8000 + (self.hdma_dest & 0x1FFF) + i;
                let value = self.read_byte(src);
                self.write_byte(dst, value);
            }
            self.hdma_source += 16;
            self.hdma_dest += 16;
        }
        
        self.hdma_active = false;
        self.hdma_length = 0xFF;
    }
    
    /// Run one block of HBlank HDMA
    pub fn step_hblank_hdma(&mut self) {
        if !self.hdma_active || !self.hdma_hblank {
            return;
        }
        
        // Transfer 16 bytes
        for i in 0..16u16 {
            let src = self.hdma_source + i;
            let dst = 0x8000 + (self.hdma_dest & 0x1FFF) + i;
            let value = self.read_byte(src);
            self.write_byte(dst, value);
        }
        
        self.hdma_source += 16;
        self.hdma_dest += 16;
        
        if self.hdma_length == 0 {
            self.hdma_active = false;
            self.hdma_length = 0xFF;
        } else {
            self.hdma_length -= 1;
        }
    }
    
    /// Request an interrupt
    pub fn request_interrupt(&mut self, flag: u8) {
        self.io[0x0F] |= flag;
    }
    
    /// Update button state from Joypad component
    pub fn update_joypad(&mut self, joypad: &Joypad) {
        self.button_state = joypad.buttons();
    }
    
    /// Get cartridge reference
    pub fn cartridge(&self) -> &Cartridge {
        &self.cartridge
    }
    
    /// Get mutable cartridge reference
    pub fn cartridge_mut(&mut self) -> &mut Cartridge {
        &mut self.cartridge
    }
    
    /// Get VRAM for PPU access
    pub fn vram(&self) -> &[u8] {
        &self.vram
    }
    
    /// Get OAM for PPU access
    pub fn oam(&self) -> &[u8; OAM_SIZE] {
        &self.oam
    }
    
    /// Get I/O registers
    pub fn io(&self) -> &[u8; IO_SIZE] {
        &self.io
    }
    
    /// Get mutable I/O registers
    pub fn io_mut(&mut self) -> &mut [u8; IO_SIZE] {
        &mut self.io
    }
    
    /// Get current state for serialization
    pub fn state(&self) -> MmuState {
        MmuState {
            vram: self.vram.clone(),
            wram: self.wram.clone(),
            oam: self.oam.to_vec(),
            hram: self.hram.to_vec(),
            io: self.io.to_vec(),
            ie: self.ie,
            vram_bank: self.vram_bank,
            wram_bank: self.wram_bank,
            dma_active: self.dma_active,
            dma_byte: self.dma_byte,
            dma_source: self.dma_source,
            hdma_active: self.hdma_active,
            hdma_source: self.hdma_source,
            hdma_dest: self.hdma_dest,
            hdma_length: self.hdma_length,
            hdma_hblank: self.hdma_hblank,
        }
    }
    
    /// Load state from serialization
    pub fn load_state(&mut self, state: MmuState) -> Result<(), String> {
        if state.vram.len() != self.vram.len() {
            return Err("VRAM size mismatch".to_string());
        }
        
        self.vram = state.vram;
        self.wram = state.wram;
        self.oam.copy_from_slice(&state.oam);
        self.hram.copy_from_slice(&state.hram);
        self.io.copy_from_slice(&state.io);
        self.ie = state.ie;
        self.vram_bank = state.vram_bank;
        self.wram_bank = state.wram_bank;
        self.dma_active = state.dma_active;
        self.dma_byte = state.dma_byte;
        self.dma_source = state.dma_source;
        self.hdma_active = state.hdma_active;
        self.hdma_source = state.hdma_source;
        self.hdma_dest = state.hdma_dest;
        self.hdma_length = state.hdma_length;
        self.hdma_hblank = state.hdma_hblank;
        
        Ok(())
    }
    
    /// Take pending audio writes and clear the queue
    pub fn take_audio_writes(&mut self) -> Vec<(u16, u8)> {
        std::mem::take(&mut self.audio_writes)
    }
}