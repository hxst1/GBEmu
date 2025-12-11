//! # Cartridge Module
//! 
//! Supports various Memory Bank Controllers:
//! - MBC0 (No MBC / ROM only)
//! - MBC1 (max 2MB ROM, 32KB RAM)
//! - MBC2 (max 256KB ROM, 512 nibbles RAM)
//! - MBC3 (max 2MB ROM, 32KB RAM, RTC)
//! - MBC5 (max 8MB ROM, 128KB RAM)

use serde::{Serialize, Deserialize};

/// Cartridge header offsets
const TITLE_START: usize = 0x0134;
const TITLE_END: usize = 0x0143;
const CGB_FLAG: usize = 0x0143;
const CARTRIDGE_TYPE: usize = 0x0147;
#[allow(dead_code)]
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;

/// MBC types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MbcType {
    None,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

/// RTC register (for MBC3)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Rtc {
    /// Seconds (0-59)
    pub seconds: u8,
    /// Minutes (0-59)
    pub minutes: u8,
    /// Hours (0-23)
    pub hours: u8,
    /// Days low (lower 8 bits)
    pub days_low: u8,
    /// Days high (bit 0 = day counter MSB, bit 6 = halt, bit 7 = day overflow)
    pub days_high: u8,
    /// Latched values
    pub latched: [u8; 5],
    /// Last latch write
    pub latch_ready: bool,
    /// Internal counter for sub-second timing
    pub sub_seconds: u32,
}

impl Rtc {
    /// Get the full day counter (0-511)
    pub fn days(&self) -> u16 {
        (self.days_low as u16) | (((self.days_high & 0x01) as u16) << 8)
    }
    
    /// Set days counter
    pub fn set_days(&mut self, days: u16) {
        self.days_low = days as u8;
        self.days_high = (self.days_high & 0xFE) | ((days >> 8) as u8 & 0x01);
    }
    
    /// Check if RTC is halted
    pub fn is_halted(&self) -> bool {
        self.days_high & 0x40 != 0
    }
    
    /// Tick the RTC (call at 1Hz when not halted)
    pub fn tick(&mut self) {
        if self.is_halted() {
            return;
        }
        
        self.seconds += 1;
        if self.seconds >= 60 {
            self.seconds = 0;
            self.minutes += 1;
            
            if self.minutes >= 60 {
                self.minutes = 0;
                self.hours += 1;
                
                if self.hours >= 24 {
                    self.hours = 0;
                    let days = self.days() + 1;
                    
                    if days >= 512 {
                        self.set_days(0);
                        // Set overflow flag
                        self.days_high |= 0x80;
                    } else {
                        self.set_days(days);
                    }
                }
            }
        }
    }
    
    /// Latch current time
    pub fn latch(&mut self) {
        self.latched[0] = self.seconds;
        self.latched[1] = self.minutes;
        self.latched[2] = self.hours;
        self.latched[3] = self.days_low;
        self.latched[4] = self.days_high;
    }
    
    /// Read latched register
    pub fn read(&self, reg: u8) -> u8 {
        match reg {
            0x08 => self.latched[0],
            0x09 => self.latched[1],
            0x0A => self.latched[2],
            0x0B => self.latched[3],
            0x0C => self.latched[4],
            _ => 0xFF,
        }
    }
    
    /// Write register
    pub fn write(&mut self, reg: u8, value: u8) {
        match reg {
            0x08 => self.seconds = value & 0x3F,
            0x09 => self.minutes = value & 0x3F,
            0x0A => self.hours = value & 0x1F,
            0x0B => self.days_low = value,
            0x0C => self.days_high = value & 0xC1,
            _ => {}
        }
    }
}

/// Cartridge state for serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeState {
    pub rom_bank: u16,
    pub ram_bank: u8,
    pub ram_enabled: bool,
    pub banking_mode: u8,
    pub ram: Vec<u8>,
    pub rtc: Option<Rtc>,
}

/// Game Boy Cartridge
pub struct Cartridge {
    /// ROM data
    rom: Vec<u8>,
    
    /// External RAM
    ram: Vec<u8>,
    
    /// Game title
    title: String,
    
    /// MBC type
    mbc_type: MbcType,
    
    /// Is CGB game
    is_cgb: bool,
    
    /// Has battery backup
    has_battery: bool,
    
    /// Has RTC (for future RTC persistence)
    #[allow(dead_code)]
    has_rtc: bool,
    
    /// Current ROM bank (14-bit for MBC5)
    rom_bank: u16,
    
    /// Current RAM bank
    ram_bank: u8,
    
    /// RAM enabled
    ram_enabled: bool,
    
    /// MBC1 banking mode (0 = ROM, 1 = RAM)
    banking_mode: u8,
    
    /// RTC for MBC3
    rtc: Option<Rtc>,
    
    /// RTC register selected
    rtc_register: u8,
}

impl Cartridge {
    /// Create a cartridge from ROM data
    pub fn from_rom(data: &[u8]) -> Result<Self, String> {
        if data.len() < 0x150 {
            return Err("ROM too small".to_string());
        }
        
        // Extract title
        let title_bytes: Vec<u8> = data[TITLE_START..TITLE_END]
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect();
        let title = String::from_utf8_lossy(&title_bytes).to_string();
        
        // Check CGB flag
        let is_cgb = data[CGB_FLAG] == 0x80 || data[CGB_FLAG] == 0xC0;
        
        // Parse cartridge type
        let cart_type = data[CARTRIDGE_TYPE];
        let (mbc_type, has_battery, has_rtc) = match cart_type {
            0x00 => (MbcType::None, false, false),
            0x01 => (MbcType::Mbc1, false, false),
            0x02 => (MbcType::Mbc1, false, false),
            0x03 => (MbcType::Mbc1, true, false),
            0x05 => (MbcType::Mbc2, false, false),
            0x06 => (MbcType::Mbc2, true, false),
            0x0F => (MbcType::Mbc3, true, true),
            0x10 => (MbcType::Mbc3, true, true),
            0x11 => (MbcType::Mbc3, false, false),
            0x12 => (MbcType::Mbc3, false, false),
            0x13 => (MbcType::Mbc3, true, false),
            0x19 => (MbcType::Mbc5, false, false),
            0x1A => (MbcType::Mbc5, false, false),
            0x1B => (MbcType::Mbc5, true, false),
            0x1C => (MbcType::Mbc5, false, false),
            0x1D => (MbcType::Mbc5, false, false),
            0x1E => (MbcType::Mbc5, true, false),
            _ => return Err(format!("Unsupported cartridge type: 0x{:02X}", cart_type)),
        };
        
        // Calculate RAM size
        let ram_size = match data[RAM_SIZE] {
            0x00 => 0,
            0x01 => 2 * 1024,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => 0,
        };
        
        // MBC2 has internal 512 nibble RAM
        let ram_size = if mbc_type == MbcType::Mbc2 { 512 } else { ram_size };
        
        Ok(Self {
            rom: data.to_vec(),
            ram: vec![0; ram_size],
            title,
            mbc_type,
            is_cgb,
            has_battery,
            has_rtc,
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: 0,
            rtc: if has_rtc { Some(Rtc::default()) } else { None },
            rtc_register: 0,
        })
    }
    
    /// Get game title
    pub fn title(&self) -> &str {
        &self.title
    }
    
    /// Check if CGB game
    pub fn is_cgb(&self) -> bool {
        self.is_cgb
    }
    
    /// Read from ROM area
    pub fn read_rom(&self, addr: u16) -> u8 {
        match self.mbc_type {
            MbcType::None => {
                self.rom.get(addr as usize).copied().unwrap_or(0xFF)
            }
            
            MbcType::Mbc1 => {
                let offset = if addr < 0x4000 {
                    // Bank 0 (or bank 0x20/0x40/0x60 in mode 1)
                    if self.banking_mode == 1 {
                        let bank = (self.ram_bank as usize & 0x03) << 5;
                        bank * 0x4000 + addr as usize
                    } else {
                        addr as usize
                    }
                } else {
                    // Bank N
                    let bank = (self.rom_bank as usize & 0x1F)
                        | ((self.ram_bank as usize & 0x03) << 5);
                    let bank = if bank & 0x1F == 0 { bank + 1 } else { bank };
                    bank * 0x4000 + (addr as usize - 0x4000)
                };
                self.rom.get(offset % self.rom.len()).copied().unwrap_or(0xFF)
            }
            
            MbcType::Mbc2 => {
                let offset = if addr < 0x4000 {
                    addr as usize
                } else {
                    let bank = (self.rom_bank as usize).max(1) & 0x0F;
                    bank * 0x4000 + (addr as usize - 0x4000)
                };
                self.rom.get(offset % self.rom.len()).copied().unwrap_or(0xFF)
            }
            
            MbcType::Mbc3 => {
                let offset = if addr < 0x4000 {
                    addr as usize
                } else {
                    let bank = (self.rom_bank as usize).max(1) & 0x7F;
                    bank * 0x4000 + (addr as usize - 0x4000)
                };
                self.rom.get(offset % self.rom.len()).copied().unwrap_or(0xFF)
            }
            
            MbcType::Mbc5 => {
                let offset = if addr < 0x4000 {
                    addr as usize
                } else {
                    let bank = self.rom_bank as usize;
                    bank * 0x4000 + (addr as usize - 0x4000)
                };
                self.rom.get(offset % self.rom.len()).copied().unwrap_or(0xFF)
            }
        }
    }
    
    /// Write to ROM area (MBC control)
    pub fn write_rom(&mut self, addr: u16, value: u8) {
        match self.mbc_type {
            MbcType::None => {}
            
            MbcType::Mbc1 => {
                match addr {
                    // RAM enable
                    0x0000..=0x1FFF => {
                        self.ram_enabled = (value & 0x0F) == 0x0A;
                    }
                    // ROM bank low bits
                    0x2000..=0x3FFF => {
                        let bank = value & 0x1F;
                        self.rom_bank = (self.rom_bank & 0x60) | bank as u16;
                    }
                    // RAM bank / ROM bank high bits
                    0x4000..=0x5FFF => {
                        self.ram_bank = value & 0x03;
                    }
                    // Banking mode
                    0x6000..=0x7FFF => {
                        self.banking_mode = value & 0x01;
                    }
                    _ => {}
                }
            }
            
            MbcType::Mbc2 => {
                match addr {
                    // RAM enable (bit 8 of address must be 0)
                    0x0000..=0x3FFF if addr & 0x0100 == 0 => {
                        self.ram_enabled = (value & 0x0F) == 0x0A;
                    }
                    // ROM bank (bit 8 of address must be 1)
                    0x0000..=0x3FFF if addr & 0x0100 != 0 => {
                        self.rom_bank = (value & 0x0F).max(1) as u16;
                    }
                    _ => {}
                }
            }
            
            MbcType::Mbc3 => {
                match addr {
                    // RAM/RTC enable
                    0x0000..=0x1FFF => {
                        self.ram_enabled = (value & 0x0F) == 0x0A;
                    }
                    // ROM bank
                    0x2000..=0x3FFF => {
                        self.rom_bank = (value & 0x7F).max(1) as u16;
                    }
                    // RAM bank / RTC register select
                    0x4000..=0x5FFF => {
                        if value <= 0x03 {
                            self.ram_bank = value;
                            self.rtc_register = 0;
                        } else if value >= 0x08 && value <= 0x0C {
                            self.rtc_register = value;
                        }
                    }
                    // Latch clock data
                    0x6000..=0x7FFF => {
                        if let Some(ref mut rtc) = self.rtc {
                            if value == 0x01 && rtc.latch_ready {
                                rtc.latch();
                            }
                            rtc.latch_ready = value == 0x00;
                        }
                    }
                    _ => {}
                }
            }
            
            MbcType::Mbc5 => {
                match addr {
                    // RAM enable
                    0x0000..=0x1FFF => {
                        self.ram_enabled = (value & 0x0F) == 0x0A;
                    }
                    // ROM bank low 8 bits
                    0x2000..=0x2FFF => {
                        self.rom_bank = (self.rom_bank & 0x100) | value as u16;
                    }
                    // ROM bank bit 8
                    0x3000..=0x3FFF => {
                        self.rom_bank = (self.rom_bank & 0xFF) | ((value as u16 & 0x01) << 8);
                    }
                    // RAM bank
                    0x4000..=0x5FFF => {
                        self.ram_bank = value & 0x0F;
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Read from RAM area
    pub fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            // Check for RTC read (MBC3)
            if self.rtc_register != 0 {
                if let Some(ref rtc) = self.rtc {
                    return rtc.read(self.rtc_register);
                }
            }
            return 0xFF;
        }
        
        match self.mbc_type {
            MbcType::None => {
                self.ram.get((addr - 0xA000) as usize).copied().unwrap_or(0xFF)
            }
            
            MbcType::Mbc1 => {
                let bank = if self.banking_mode == 1 {
                    self.ram_bank as usize & 0x03
                } else {
                    0
                };
                let offset = bank * 0x2000 + (addr as usize - 0xA000);
                self.ram.get(offset % self.ram.len()).copied().unwrap_or(0xFF)
            }
            
            MbcType::Mbc2 => {
                // MBC2 only has 512 nibbles (only lower 4 bits valid)
                let offset = (addr as usize - 0xA000) & 0x1FF;
                self.ram.get(offset).map(|&v| v | 0xF0).unwrap_or(0xFF)
            }
            
            MbcType::Mbc3 => {
                if self.rtc_register != 0 {
                    if let Some(ref rtc) = self.rtc {
                        return rtc.read(self.rtc_register);
                    }
                }
                let bank = self.ram_bank as usize & 0x03;
                let offset = bank * 0x2000 + (addr as usize - 0xA000);
                self.ram.get(offset % self.ram.len()).copied().unwrap_or(0xFF)
            }
            
            MbcType::Mbc5 => {
                let bank = self.ram_bank as usize & 0x0F;
                let offset = bank * 0x2000 + (addr as usize - 0xA000);
                self.ram.get(offset % self.ram.len()).copied().unwrap_or(0xFF)
            }
        }
    }
    
    /// Write to RAM area
    pub fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        
        // Check for RTC write (MBC3)
        if self.rtc_register != 0 {
            if let Some(ref mut rtc) = self.rtc {
                rtc.write(self.rtc_register, value);
                return;
            }
        }
        
        if self.ram.is_empty() {
            return;
        }
        
        match self.mbc_type {
            MbcType::None => {
                if let Some(byte) = self.ram.get_mut((addr - 0xA000) as usize) {
                    *byte = value;
                }
            }
            
            MbcType::Mbc1 => {
                let bank = if self.banking_mode == 1 {
                    self.ram_bank as usize & 0x03
                } else {
                    0
                };
                let offset = bank * 0x2000 + (addr as usize - 0xA000);
                let len = self.ram.len();
                if let Some(byte) = self.ram.get_mut(offset % len) {
                    *byte = value;
                }
            }
            
            MbcType::Mbc2 => {
                let offset = (addr as usize - 0xA000) & 0x1FF;
                if let Some(byte) = self.ram.get_mut(offset) {
                    *byte = value & 0x0F;
                }
            }
            
            MbcType::Mbc3 => {
                let bank = self.ram_bank as usize & 0x03;
                let offset = bank * 0x2000 + (addr as usize - 0xA000);
                let len = self.ram.len();
                if let Some(byte) = self.ram.get_mut(offset % len) {
                    *byte = value;
                }
            }
            
            MbcType::Mbc5 => {
                let bank = self.ram_bank as usize & 0x0F;
                let offset = bank * 0x2000 + (addr as usize - 0xA000);
                let len = self.ram.len();
                if let Some(byte) = self.ram.get_mut(offset % len) {
                    *byte = value;
                }
            }
        }
    }
    
    /// Tick RTC (call at appropriate intervals)
    pub fn tick_rtc(&mut self, cycles: u32) {
        if let Some(ref mut rtc) = self.rtc {
            // Accumulate sub-second cycles
            rtc.sub_seconds += cycles;
            
            // CPU runs at 4.194304 MHz
            // Tick once per second
            if rtc.sub_seconds >= 4_194_304 {
                rtc.sub_seconds -= 4_194_304;
                rtc.tick();
            }
        }
    }
    
    /// Save RAM (for battery backup)
    pub fn save_ram(&self) -> Option<Vec<u8>> {
        if !self.has_battery || self.ram.is_empty() {
            return None;
        }
        
        let mut data = self.ram.clone();
        
        // Include RTC state if present
        if let Some(ref rtc) = self.rtc {
            // Append RTC data (48 bytes for compatibility with other emulators)
            let rtc_data = [
                rtc.seconds as u32,
                rtc.minutes as u32,
                rtc.hours as u32,
                rtc.days_low as u32,
                rtc.days_high as u32,
                rtc.latched[0] as u32,
                rtc.latched[1] as u32,
                rtc.latched[2] as u32,
                rtc.latched[3] as u32,
                rtc.latched[4] as u32,
                // Unix timestamp placeholder
                0,
                0,
            ];
            
            for val in rtc_data {
                data.extend_from_slice(&val.to_le_bytes());
            }
        }
        
        Some(data)
    }
    
    /// Load RAM (for battery backup)
    pub fn load_ram(&mut self, data: &[u8]) -> Result<(), String> {
        if self.ram.is_empty() {
            return Ok(());
        }
        
        let ram_size = self.ram.len();
        
        if data.len() < ram_size {
            return Err("Save data too small".to_string());
        }
        
        self.ram.copy_from_slice(&data[..ram_size]);
        
        // Load RTC state if present
        if let Some(ref mut rtc) = self.rtc {
            if data.len() >= ram_size + 48 {
                let rtc_offset = ram_size;
                let read_u32 = |offset: usize| {
                    u32::from_le_bytes([
                        data[rtc_offset + offset],
                        data[rtc_offset + offset + 1],
                        data[rtc_offset + offset + 2],
                        data[rtc_offset + offset + 3],
                    ]) as u8
                };
                
                rtc.seconds = read_u32(0);
                rtc.minutes = read_u32(4);
                rtc.hours = read_u32(8);
                rtc.days_low = read_u32(12);
                rtc.days_high = read_u32(16);
                rtc.latched[0] = read_u32(20);
                rtc.latched[1] = read_u32(24);
                rtc.latched[2] = read_u32(28);
                rtc.latched[3] = read_u32(32);
                rtc.latched[4] = read_u32(36);
            }
        }
        
        Ok(())
    }
    
    /// Get state for serialization
    pub fn state(&self) -> CartridgeState {
        CartridgeState {
            rom_bank: self.rom_bank,
            ram_bank: self.ram_bank,
            ram_enabled: self.ram_enabled,
            banking_mode: self.banking_mode,
            ram: self.ram.clone(),
            rtc: self.rtc.clone(),
        }
    }
    
    /// Load state
    pub fn load_state(&mut self, state: CartridgeState) {
        self.rom_bank = state.rom_bank;
        self.ram_bank = state.ram_bank;
        self.ram_enabled = state.ram_enabled;
        self.banking_mode = state.banking_mode;
        self.ram = state.ram;
        self.rtc = state.rtc;
    }
}