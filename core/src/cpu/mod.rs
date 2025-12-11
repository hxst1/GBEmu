//! # CPU Module - Sharp LR35902
//! 
//! Complete implementation of the Game Boy CPU with all instructions
//! and cycle-accurate timing.

mod instructions;
mod cb_instructions;

use crate::mmu::Mmu;
use crate::GbModel;
use serde::{Serialize, Deserialize};
use bitflags::bitflags;

bitflags! {
    /// CPU Flags register (F)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        /// Zero flag - set when result is zero
        const Z = 0b1000_0000;
        /// Subtract flag - set for subtraction operations
        const N = 0b0100_0000;
        /// Half-carry flag - set on carry from bit 3
        const H = 0b0010_0000;
        /// Carry flag - set on carry from bit 7
        const C = 0b0001_0000;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Flags::empty()
    }
}

// Manual serde implementation for Flags
impl Serialize for Flags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Flags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        Ok(Flags::from_bits_truncate(bits))
    }
}

/// CPU Registers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registers {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0,
            f: Flags::empty(),
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
}

impl Registers {
    /// Get AF register pair
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.bits() as u16)
    }
    
    /// Set AF register pair
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        // Only upper 4 bits of F are writable
        self.f = Flags::from_bits_truncate((value & 0xF0) as u8);
    }
    
    /// Get BC register pair
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    
    /// Set BC register pair
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }
    
    /// Get DE register pair
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    
    /// Set DE register pair
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }
    
    /// Get HL register pair
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }
    
    /// Set HL register pair
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
}

/// CPU state for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    pub registers: Registers,
    pub ime: bool,
    pub ime_scheduled: bool,
    pub halted: bool,
    pub stopped: bool,
    pub halt_bug: bool,
}

/// Sharp LR35902 CPU
pub struct Cpu {
    /// CPU registers
    pub regs: Registers,
    
    /// Interrupt Master Enable flag
    pub ime: bool,
    
    /// IME will be enabled after next instruction (for EI delay)
    pub ime_scheduled: bool,
    
    /// CPU is halted (waiting for interrupt)
    pub halted: bool,
    
    /// CPU is stopped (low power mode)
    pub stopped: bool,
    
    /// HALT bug active (PC not incremented on next instruction)
    pub halt_bug: bool,
}

impl Cpu {
    /// Create a new CPU
    pub fn new() -> Self {
        Self {
            regs: Registers::default(),
            ime: false,
            ime_scheduled: false,
            halted: false,
            stopped: false,
            halt_bug: false,
        }
    }
    
    /// Reset CPU to initial state
    pub fn reset(&mut self) {
        self.regs = Registers::default();
        self.ime = false;
        self.ime_scheduled = false;
        self.halted = false;
        self.stopped = false;
        self.halt_bug = false;
    }
    
    /// Initialize registers based on Game Boy model
    pub fn init_for_model(&mut self, model: GbModel) {
        match model {
            GbModel::Dmg | GbModel::Pocket => {
                // DMG boot ROM leaves these values
                self.regs.a = 0x01;
                self.regs.f = Flags::Z | Flags::H | Flags::C;
                self.regs.b = 0x00;
                self.regs.c = 0x13;
                self.regs.d = 0x00;
                self.regs.e = 0xD8;
                self.regs.h = 0x01;
                self.regs.l = 0x4D;
                self.regs.sp = 0xFFFE;
                self.regs.pc = 0x0100;
            }
            GbModel::Cgb | GbModel::CgbDmg => {
                // CGB boot ROM leaves these values
                self.regs.a = 0x11;
                self.regs.f = Flags::Z;
                self.regs.b = 0x00;
                self.regs.c = 0x00;
                self.regs.d = 0xFF;
                self.regs.e = 0x56;
                self.regs.h = 0x00;
                self.regs.l = 0x0D;
                self.regs.sp = 0xFFFE;
                self.regs.pc = 0x0100;
            }
        }
    }
    
    /// Execute one instruction and return cycles consumed
    pub fn step(&mut self, mmu: &mut Mmu) -> u32 {
        // Handle scheduled IME enable
        if self.ime_scheduled {
            self.ime_scheduled = false;
            self.ime = true;
        }
        
        // Check for interrupts
        if let Some(cycles) = self.handle_interrupts(mmu) {
            return cycles;
        }
        
        // If halted, return 4 cycles (one M-cycle)
        if self.halted {
            return 4;
        }
        
        // If stopped, return 4 cycles
        if self.stopped {
            // Check if any button pressed to exit STOP
            if mmu.read_byte(0xFF00) & 0x0F != 0x0F {
                self.stopped = false;
            }
            return 4;
        }
        
        // Fetch opcode
        let opcode = self.fetch_byte(mmu);
        
        // Execute instruction
        self.execute(opcode, mmu)
    }
    
    /// Handle pending interrupts
    fn handle_interrupts(&mut self, mmu: &mut Mmu) -> Option<u32> {
        let ie = mmu.read_byte(0xFFFF); // Interrupt Enable
        let if_ = mmu.read_byte(0xFF0F); // Interrupt Flag
        let pending = ie & if_;
        
        if pending == 0 {
            return None;
        }
        
        // Wake from HALT even if IME is disabled
        if self.halted {
            self.halted = false;
            // If IME is disabled, the HALT bug may trigger
            if !self.ime {
                self.halt_bug = true;
            }
        }
        
        // Only service interrupt if IME is enabled
        if !self.ime {
            return None;
        }
        
        // Find highest priority interrupt (bit 0 is highest)
        let interrupt_bit = pending.trailing_zeros();
        if interrupt_bit >= 5 {
            return None;
        }
        
        // Disable IME
        self.ime = false;
        
        // Clear interrupt flag
        let new_if = if_ & !(1 << interrupt_bit);
        mmu.write_byte(0xFF0F, new_if);
        
        // Push PC onto stack
        self.push_word(mmu, self.regs.pc);
        
        // Jump to interrupt vector
        let vector = match interrupt_bit {
            0 => 0x0040, // VBlank
            1 => 0x0048, // LCD STAT
            2 => 0x0050, // Timer
            3 => 0x0058, // Serial
            4 => 0x0060, // Joypad
            _ => unreachable!(),
        };
        self.regs.pc = vector;
        
        // Interrupt handling takes 20 cycles
        Some(20)
    }
    
    /// Fetch byte at PC and increment PC
    fn fetch_byte(&mut self, mmu: &Mmu) -> u8 {
        let byte = mmu.read_byte(self.regs.pc);
        
        // Handle HALT bug - PC not incremented
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.regs.pc = self.regs.pc.wrapping_add(1);
        }
        
        byte
    }
    
    /// Fetch word at PC and increment PC by 2
    fn fetch_word(&mut self, mmu: &Mmu) -> u16 {
        let low = self.fetch_byte(mmu);
        let high = self.fetch_byte(mmu);
        u16::from_le_bytes([low, high])
    }
    
    /// Push word onto stack
    fn push_word(&mut self, mmu: &mut Mmu, value: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        mmu.write_byte(self.regs.sp, (value >> 8) as u8);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        mmu.write_byte(self.regs.sp, value as u8);
    }
    
    /// Pop word from stack
    fn pop_word(&mut self, mmu: &Mmu) -> u16 {
        let low = mmu.read_byte(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        let high = mmu.read_byte(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        u16::from_le_bytes([low, high])
    }
    
    /// Get current state for serialization
    pub fn state(&self) -> CpuState {
        CpuState {
            registers: self.regs.clone(),
            ime: self.ime,
            ime_scheduled: self.ime_scheduled,
            halted: self.halted,
            stopped: self.stopped,
            halt_bug: self.halt_bug,
        }
    }
    
    /// Load state from serialization
    pub fn load_state(&mut self, state: CpuState) {
        self.regs = state.registers;
        self.ime = state.ime;
        self.ime_scheduled = state.ime_scheduled;
        self.halted = state.halted;
        self.stopped = state.stopped;
        self.halt_bug = state.halt_bug;
    }
    
    // ========== ALU Operations ==========
    
    /// Add with carry
    fn adc(&mut self, value: u8) {
        let carry = if self.regs.f.contains(Flags::C) { 1u8 } else { 0 };
        let a = self.regs.a;
        let result = a.wrapping_add(value).wrapping_add(carry);
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if (a & 0x0F) + (value & 0x0F) + carry > 0x0F {
            self.regs.f |= Flags::H;
        }
        if (a as u16) + (value as u16) + (carry as u16) > 0xFF {
            self.regs.f |= Flags::C;
        }
        
        self.regs.a = result;
    }
    
    /// Add without carry
    fn add(&mut self, value: u8) {
        let a = self.regs.a;
        let result = a.wrapping_add(value);
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if (a & 0x0F) + (value & 0x0F) > 0x0F {
            self.regs.f |= Flags::H;
        }
        if (a as u16) + (value as u16) > 0xFF {
            self.regs.f |= Flags::C;
        }
        
        self.regs.a = result;
    }
    
    /// Add to HL (16-bit)
    fn add_hl(&mut self, value: u16) {
        let hl = self.regs.hl();
        let result = hl.wrapping_add(value);
        
        self.regs.f.remove(Flags::N);
        if (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF {
            self.regs.f |= Flags::H;
        } else {
            self.regs.f.remove(Flags::H);
        }
        if (hl as u32) + (value as u32) > 0xFFFF {
            self.regs.f |= Flags::C;
        } else {
            self.regs.f.remove(Flags::C);
        }
        
        self.regs.set_hl(result);
    }
    
    /// Add signed value to SP
    fn add_sp(&mut self, value: i8) -> u16 {
        let sp = self.regs.sp;
        let offset = value as i16 as u16;
        let result = sp.wrapping_add(offset);
        
        self.regs.f = Flags::empty();
        if (sp & 0x0F) + (offset & 0x0F) > 0x0F {
            self.regs.f |= Flags::H;
        }
        if (sp & 0xFF) + (offset & 0xFF) > 0xFF {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Bitwise AND
    fn and(&mut self, value: u8) {
        self.regs.a &= value;
        
        self.regs.f = Flags::H;
        if self.regs.a == 0 {
            self.regs.f |= Flags::Z;
        }
    }
    
    /// Compare (subtract without storing result)
    fn cp(&mut self, value: u8) {
        let a = self.regs.a;
        let result = a.wrapping_sub(value);
        
        self.regs.f = Flags::N;
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if (a & 0x0F) < (value & 0x0F) {
            self.regs.f |= Flags::H;
        }
        if a < value {
            self.regs.f |= Flags::C;
        }
    }
    
    /// Decrement 8-bit value
    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        
        self.regs.f.insert(Flags::N);
        if result == 0 {
            self.regs.f.insert(Flags::Z);
        } else {
            self.regs.f.remove(Flags::Z);
        }
        if value & 0x0F == 0 {
            self.regs.f.insert(Flags::H);
        } else {
            self.regs.f.remove(Flags::H);
        }
        
        result
    }
    
    /// Increment 8-bit value
    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        
        self.regs.f.remove(Flags::N);
        if result == 0 {
            self.regs.f.insert(Flags::Z);
        } else {
            self.regs.f.remove(Flags::Z);
        }
        if value & 0x0F == 0x0F {
            self.regs.f.insert(Flags::H);
        } else {
            self.regs.f.remove(Flags::H);
        }
        
        result
    }
    
    /// Bitwise OR
    fn or(&mut self, value: u8) {
        self.regs.a |= value;
        
        self.regs.f = Flags::empty();
        if self.regs.a == 0 {
            self.regs.f |= Flags::Z;
        }
    }
    
    /// Subtract with carry
    fn sbc(&mut self, value: u8) {
        let carry = if self.regs.f.contains(Flags::C) { 1u8 } else { 0 };
        let a = self.regs.a;
        let result = a.wrapping_sub(value).wrapping_sub(carry);
        
        self.regs.f = Flags::N;
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if (a & 0x0F) < (value & 0x0F) + carry {
            self.regs.f |= Flags::H;
        }
        if (a as u16) < (value as u16) + (carry as u16) {
            self.regs.f |= Flags::C;
        }
        
        self.regs.a = result;
    }
    
    /// Subtract without carry
    fn sub(&mut self, value: u8) {
        let a = self.regs.a;
        let result = a.wrapping_sub(value);
        
        self.regs.f = Flags::N;
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if (a & 0x0F) < (value & 0x0F) {
            self.regs.f |= Flags::H;
        }
        if a < value {
            self.regs.f |= Flags::C;
        }
        
        self.regs.a = result;
    }
    
    /// Bitwise XOR
    fn xor(&mut self, value: u8) {
        self.regs.a ^= value;
        
        self.regs.f = Flags::empty();
        if self.regs.a == 0 {
            self.regs.f |= Flags::Z;
        }
    }
    
    // ========== Rotate/Shift Operations ==========
    
    /// Rotate left through carry
    fn rl(&mut self, value: u8) -> u8 {
        let carry = if self.regs.f.contains(Flags::C) { 1 } else { 0 };
        let result = (value << 1) | carry;
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if value & 0x80 != 0 {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Rotate left (circular)
    fn rlc(&mut self, value: u8) -> u8 {
        let result = value.rotate_left(1);
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if value & 0x80 != 0 {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Rotate right through carry
    fn rr(&mut self, value: u8) -> u8 {
        let carry = if self.regs.f.contains(Flags::C) { 0x80 } else { 0 };
        let result = (value >> 1) | carry;
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if value & 0x01 != 0 {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Rotate right (circular)
    fn rrc(&mut self, value: u8) -> u8 {
        let result = value.rotate_right(1);
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if value & 0x01 != 0 {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Shift left arithmetic
    fn sla(&mut self, value: u8) -> u8 {
        let result = value << 1;
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if value & 0x80 != 0 {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Shift right arithmetic (preserves sign)
    fn sra(&mut self, value: u8) -> u8 {
        let result = (value >> 1) | (value & 0x80);
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if value & 0x01 != 0 {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Shift right logical
    fn srl(&mut self, value: u8) -> u8 {
        let result = value >> 1;
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        if value & 0x01 != 0 {
            self.regs.f |= Flags::C;
        }
        
        result
    }
    
    /// Swap nibbles
    fn swap(&mut self, value: u8) -> u8 {
        let result = (value >> 4) | (value << 4);
        
        self.regs.f = Flags::empty();
        if result == 0 {
            self.regs.f |= Flags::Z;
        }
        
        result
    }
    
    /// Test bit
    fn bit(&mut self, bit: u8, value: u8) {
        self.regs.f.remove(Flags::N);
        self.regs.f.insert(Flags::H);
        
        if value & (1 << bit) == 0 {
            self.regs.f.insert(Flags::Z);
        } else {
            self.regs.f.remove(Flags::Z);
        }
    }
    
    /// Reset bit
    fn res(&self, bit: u8, value: u8) -> u8 {
        value & !(1 << bit)
    }
    
    /// Set bit
    fn set(&self, bit: u8, value: u8) -> u8 {
        value | (1 << bit)
    }
}
