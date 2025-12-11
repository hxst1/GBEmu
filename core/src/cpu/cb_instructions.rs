//! CB-prefixed instructions (bit operations, rotates, shifts)

use super::Cpu;
use crate::mmu::Mmu;

impl Cpu {
    /// Execute a CB-prefixed instruction
    pub fn execute_cb(&mut self, opcode: u8, mmu: &mut Mmu) -> u32 {
        // CB instructions follow a pattern:
        // Bits 7-6: operation type
        // Bits 5-3: bit number (for BIT/RES/SET) or sub-operation
        // Bits 2-0: register (B,C,D,E,H,L,(HL),A)
        
        let reg = opcode & 0x07;
        let bit = (opcode >> 3) & 0x07;
        
        match opcode {
            // ========== RLC r8 ==========
            0x00..=0x07 => {
                let value = self.get_reg8(reg, mmu);
                let result = self.rlc(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== RRC r8 ==========
            0x08..=0x0F => {
                let value = self.get_reg8(reg, mmu);
                let result = self.rrc(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== RL r8 ==========
            0x10..=0x17 => {
                let value = self.get_reg8(reg, mmu);
                let result = self.rl(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== RR r8 ==========
            0x18..=0x1F => {
                let value = self.get_reg8(reg, mmu);
                let result = self.rr(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== SLA r8 ==========
            0x20..=0x27 => {
                let value = self.get_reg8(reg, mmu);
                let result = self.sla(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== SRA r8 ==========
            0x28..=0x2F => {
                let value = self.get_reg8(reg, mmu);
                let result = self.sra(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== SWAP r8 ==========
            0x30..=0x37 => {
                let value = self.get_reg8(reg, mmu);
                let result = self.swap(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== SRL r8 ==========
            0x38..=0x3F => {
                let value = self.get_reg8(reg, mmu);
                let result = self.srl(value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== BIT b, r8 ==========
            0x40..=0x7F => {
                let value = self.get_reg8(reg, mmu);
                self.bit(bit, value);
                if reg == 6 { 12 } else { 8 }
            }
            
            // ========== RES b, r8 ==========
            0x80..=0xBF => {
                let value = self.get_reg8(reg, mmu);
                let result = self.res(bit, value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
            
            // ========== SET b, r8 ==========
            0xC0..=0xFF => {
                let value = self.get_reg8(reg, mmu);
                let result = self.set(bit, value);
                self.set_reg8(reg, result, mmu);
                if reg == 6 { 16 } else { 8 }
            }
        }
    }
    
    /// Get value from register by index
    /// 0=B, 1=C, 2=D, 3=E, 4=H, 5=L, 6=(HL), 7=A
    fn get_reg8(&self, reg: u8, mmu: &Mmu) -> u8 {
        match reg {
            0 => self.regs.b,
            1 => self.regs.c,
            2 => self.regs.d,
            3 => self.regs.e,
            4 => self.regs.h,
            5 => self.regs.l,
            6 => mmu.read_byte(self.regs.hl()),
            7 => self.regs.a,
            _ => unreachable!(),
        }
    }
    
    /// Set value to register by index
    fn set_reg8(&mut self, reg: u8, value: u8, mmu: &mut Mmu) {
        match reg {
            0 => self.regs.b = value,
            1 => self.regs.c = value,
            2 => self.regs.d = value,
            3 => self.regs.e = value,
            4 => self.regs.h = value,
            5 => self.regs.l = value,
            6 => mmu.write_byte(self.regs.hl(), value),
            7 => self.regs.a = value,
            _ => unreachable!(),
        }
    }
}
