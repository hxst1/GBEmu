//! Main instruction set implementation for LR35902

use super::{Cpu, Flags};
use crate::mmu::Mmu;

impl Cpu {
    /// Execute a single instruction and return cycles consumed
    pub fn execute(&mut self, opcode: u8, mmu: &mut Mmu) -> u32 {
        match opcode {
            // ========== NOP ==========
            0x00 => 4, // NOP
            
            // ========== LD r16, nn ==========
            0x01 => { // LD BC, nn
                let value = self.fetch_word(mmu);
                self.regs.set_bc(value);
                12
            }
            0x11 => { // LD DE, nn
                let value = self.fetch_word(mmu);
                self.regs.set_de(value);
                12
            }
            0x21 => { // LD HL, nn
                let value = self.fetch_word(mmu);
                self.regs.set_hl(value);
                12
            }
            0x31 => { // LD SP, nn
                self.regs.sp = self.fetch_word(mmu);
                12
            }
            
            // ========== LD (r16), A ==========
            0x02 => { // LD (BC), A
                mmu.write_byte(self.regs.bc(), self.regs.a);
                8
            }
            0x12 => { // LD (DE), A
                mmu.write_byte(self.regs.de(), self.regs.a);
                8
            }
            0x22 => { // LD (HL+), A
                let hl = self.regs.hl();
                mmu.write_byte(hl, self.regs.a);
                self.regs.set_hl(hl.wrapping_add(1));
                8
            }
            0x32 => { // LD (HL-), A
                let hl = self.regs.hl();
                mmu.write_byte(hl, self.regs.a);
                self.regs.set_hl(hl.wrapping_sub(1));
                8
            }
            
            // ========== INC r16 ==========
            0x03 => { // INC BC
                self.regs.set_bc(self.regs.bc().wrapping_add(1));
                8
            }
            0x13 => { // INC DE
                self.regs.set_de(self.regs.de().wrapping_add(1));
                8
            }
            0x23 => { // INC HL
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
                8
            }
            0x33 => { // INC SP
                self.regs.sp = self.regs.sp.wrapping_add(1);
                8
            }
            
            // ========== INC r8 ==========
            0x04 => { self.regs.b = self.inc(self.regs.b); 4 }
            0x0C => { self.regs.c = self.inc(self.regs.c); 4 }
            0x14 => { self.regs.d = self.inc(self.regs.d); 4 }
            0x1C => { self.regs.e = self.inc(self.regs.e); 4 }
            0x24 => { self.regs.h = self.inc(self.regs.h); 4 }
            0x2C => { self.regs.l = self.inc(self.regs.l); 4 }
            0x34 => { // INC (HL)
                let addr = self.regs.hl();
                let value = self.inc(mmu.read_byte(addr));
                mmu.write_byte(addr, value);
                12
            }
            0x3C => { self.regs.a = self.inc(self.regs.a); 4 }
            
            // ========== DEC r8 ==========
            0x05 => { self.regs.b = self.dec(self.regs.b); 4 }
            0x0D => { self.regs.c = self.dec(self.regs.c); 4 }
            0x15 => { self.regs.d = self.dec(self.regs.d); 4 }
            0x1D => { self.regs.e = self.dec(self.regs.e); 4 }
            0x25 => { self.regs.h = self.dec(self.regs.h); 4 }
            0x2D => { self.regs.l = self.dec(self.regs.l); 4 }
            0x35 => { // DEC (HL)
                let addr = self.regs.hl();
                let value = self.dec(mmu.read_byte(addr));
                mmu.write_byte(addr, value);
                12
            }
            0x3D => { self.regs.a = self.dec(self.regs.a); 4 }
            
            // ========== LD r8, n ==========
            0x06 => { self.regs.b = self.fetch_byte(mmu); 8 }
            0x0E => { self.regs.c = self.fetch_byte(mmu); 8 }
            0x16 => { self.regs.d = self.fetch_byte(mmu); 8 }
            0x1E => { self.regs.e = self.fetch_byte(mmu); 8 }
            0x26 => { self.regs.h = self.fetch_byte(mmu); 8 }
            0x2E => { self.regs.l = self.fetch_byte(mmu); 8 }
            0x36 => { // LD (HL), n
                let value = self.fetch_byte(mmu);
                mmu.write_byte(self.regs.hl(), value);
                12
            }
            0x3E => { self.regs.a = self.fetch_byte(mmu); 8 }
            
            // ========== Rotate A instructions ==========
            0x07 => { // RLCA
                self.regs.a = self.rlc(self.regs.a);
                self.regs.f.remove(Flags::Z);
                4
            }
            0x0F => { // RRCA
                self.regs.a = self.rrc(self.regs.a);
                self.regs.f.remove(Flags::Z);
                4
            }
            0x17 => { // RLA
                self.regs.a = self.rl(self.regs.a);
                self.regs.f.remove(Flags::Z);
                4
            }
            0x1F => { // RRA
                self.regs.a = self.rr(self.regs.a);
                self.regs.f.remove(Flags::Z);
                4
            }
            
            // ========== LD (nn), SP ==========
            0x08 => {
                let addr = self.fetch_word(mmu);
                mmu.write_byte(addr, self.regs.sp as u8);
                mmu.write_byte(addr.wrapping_add(1), (self.regs.sp >> 8) as u8);
                20
            }
            
            // ========== ADD HL, r16 ==========
            0x09 => { self.add_hl(self.regs.bc()); 8 }
            0x19 => { self.add_hl(self.regs.de()); 8 }
            0x29 => { self.add_hl(self.regs.hl()); 8 }
            0x39 => { self.add_hl(self.regs.sp); 8 }
            
            // ========== LD A, (r16) ==========
            0x0A => { // LD A, (BC)
                self.regs.a = mmu.read_byte(self.regs.bc());
                8
            }
            0x1A => { // LD A, (DE)
                self.regs.a = mmu.read_byte(self.regs.de());
                8
            }
            0x2A => { // LD A, (HL+)
                let hl = self.regs.hl();
                self.regs.a = mmu.read_byte(hl);
                self.regs.set_hl(hl.wrapping_add(1));
                8
            }
            0x3A => { // LD A, (HL-)
                let hl = self.regs.hl();
                self.regs.a = mmu.read_byte(hl);
                self.regs.set_hl(hl.wrapping_sub(1));
                8
            }
            
            // ========== DEC r16 ==========
            0x0B => { self.regs.set_bc(self.regs.bc().wrapping_sub(1)); 8 }
            0x1B => { self.regs.set_de(self.regs.de().wrapping_sub(1)); 8 }
            0x2B => { self.regs.set_hl(self.regs.hl().wrapping_sub(1)); 8 }
            0x3B => { self.regs.sp = self.regs.sp.wrapping_sub(1); 8 }
            
            // ========== STOP ==========
            0x10 => {
                self.fetch_byte(mmu); // consume next byte
                self.stopped = true;
                4
            }
            
            // ========== JR e ==========
            0x18 => { // JR e
                let offset = self.fetch_byte(mmu) as i8;
                self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                12
            }
            0x20 => { // JR NZ, e
                let offset = self.fetch_byte(mmu) as i8;
                if !self.regs.f.contains(Flags::Z) {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else {
                    8
                }
            }
            0x28 => { // JR Z, e
                let offset = self.fetch_byte(mmu) as i8;
                if self.regs.f.contains(Flags::Z) {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else {
                    8
                }
            }
            0x30 => { // JR NC, e
                let offset = self.fetch_byte(mmu) as i8;
                if !self.regs.f.contains(Flags::C) {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else {
                    8
                }
            }
            0x38 => { // JR C, e
                let offset = self.fetch_byte(mmu) as i8;
                if self.regs.f.contains(Flags::C) {
                    self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
                    12
                } else {
                    8
                }
            }
            
            // ========== DAA ==========
            0x27 => {
                let mut adjust = 0u8;
                let mut carry = false;
                
                if self.regs.f.contains(Flags::H) || 
                   (!self.regs.f.contains(Flags::N) && (self.regs.a & 0x0F) > 9) {
                    adjust |= 0x06;
                }
                
                if self.regs.f.contains(Flags::C) || 
                   (!self.regs.f.contains(Flags::N) && self.regs.a > 0x99) {
                    adjust |= 0x60;
                    carry = true;
                }
                
                if self.regs.f.contains(Flags::N) {
                    self.regs.a = self.regs.a.wrapping_sub(adjust);
                } else {
                    self.regs.a = self.regs.a.wrapping_add(adjust);
                }
                
                self.regs.f.remove(Flags::H);
                if self.regs.a == 0 {
                    self.regs.f.insert(Flags::Z);
                } else {
                    self.regs.f.remove(Flags::Z);
                }
                if carry {
                    self.regs.f.insert(Flags::C);
                } else {
                    self.regs.f.remove(Flags::C);
                }
                
                4
            }
            
            // ========== CPL ==========
            0x2F => {
                self.regs.a = !self.regs.a;
                self.regs.f.insert(Flags::N | Flags::H);
                4
            }
            
            // ========== SCF ==========
            0x37 => {
                self.regs.f.remove(Flags::N | Flags::H);
                self.regs.f.insert(Flags::C);
                4
            }
            
            // ========== CCF ==========
            0x3F => {
                self.regs.f.remove(Flags::N | Flags::H);
                self.regs.f.toggle(Flags::C);
                4
            }
            
            // ========== LD r8, r8 ==========
            // B
            0x40 => { 4 } // LD B, B (NOP)
            0x41 => { self.regs.b = self.regs.c; 4 }
            0x42 => { self.regs.b = self.regs.d; 4 }
            0x43 => { self.regs.b = self.regs.e; 4 }
            0x44 => { self.regs.b = self.regs.h; 4 }
            0x45 => { self.regs.b = self.regs.l; 4 }
            0x46 => { self.regs.b = mmu.read_byte(self.regs.hl()); 8 }
            0x47 => { self.regs.b = self.regs.a; 4 }
            // C
            0x48 => { self.regs.c = self.regs.b; 4 }
            0x49 => { 4 } // LD C, C (NOP)
            0x4A => { self.regs.c = self.regs.d; 4 }
            0x4B => { self.regs.c = self.regs.e; 4 }
            0x4C => { self.regs.c = self.regs.h; 4 }
            0x4D => { self.regs.c = self.regs.l; 4 }
            0x4E => { self.regs.c = mmu.read_byte(self.regs.hl()); 8 }
            0x4F => { self.regs.c = self.regs.a; 4 }
            // D
            0x50 => { self.regs.d = self.regs.b; 4 }
            0x51 => { self.regs.d = self.regs.c; 4 }
            0x52 => { 4 } // LD D, D (NOP)
            0x53 => { self.regs.d = self.regs.e; 4 }
            0x54 => { self.regs.d = self.regs.h; 4 }
            0x55 => { self.regs.d = self.regs.l; 4 }
            0x56 => { self.regs.d = mmu.read_byte(self.regs.hl()); 8 }
            0x57 => { self.regs.d = self.regs.a; 4 }
            // E
            0x58 => { self.regs.e = self.regs.b; 4 }
            0x59 => { self.regs.e = self.regs.c; 4 }
            0x5A => { self.regs.e = self.regs.d; 4 }
            0x5B => { 4 } // LD E, E (NOP)
            0x5C => { self.regs.e = self.regs.h; 4 }
            0x5D => { self.regs.e = self.regs.l; 4 }
            0x5E => { self.regs.e = mmu.read_byte(self.regs.hl()); 8 }
            0x5F => { self.regs.e = self.regs.a; 4 }
            // H
            0x60 => { self.regs.h = self.regs.b; 4 }
            0x61 => { self.regs.h = self.regs.c; 4 }
            0x62 => { self.regs.h = self.regs.d; 4 }
            0x63 => { self.regs.h = self.regs.e; 4 }
            0x64 => { 4 } // LD H, H (NOP)
            0x65 => { self.regs.h = self.regs.l; 4 }
            0x66 => { self.regs.h = mmu.read_byte(self.regs.hl()); 8 }
            0x67 => { self.regs.h = self.regs.a; 4 }
            // L
            0x68 => { self.regs.l = self.regs.b; 4 }
            0x69 => { self.regs.l = self.regs.c; 4 }
            0x6A => { self.regs.l = self.regs.d; 4 }
            0x6B => { self.regs.l = self.regs.e; 4 }
            0x6C => { self.regs.l = self.regs.h; 4 }
            0x6D => { 4 } // LD L, L (NOP)
            0x6E => { self.regs.l = mmu.read_byte(self.regs.hl()); 8 }
            0x6F => { self.regs.l = self.regs.a; 4 }
            // (HL)
            0x70 => { mmu.write_byte(self.regs.hl(), self.regs.b); 8 }
            0x71 => { mmu.write_byte(self.regs.hl(), self.regs.c); 8 }
            0x72 => { mmu.write_byte(self.regs.hl(), self.regs.d); 8 }
            0x73 => { mmu.write_byte(self.regs.hl(), self.regs.e); 8 }
            0x74 => { mmu.write_byte(self.regs.hl(), self.regs.h); 8 }
            0x75 => { mmu.write_byte(self.regs.hl(), self.regs.l); 8 }
            0x77 => { mmu.write_byte(self.regs.hl(), self.regs.a); 8 }
            // A
            0x78 => { self.regs.a = self.regs.b; 4 }
            0x79 => { self.regs.a = self.regs.c; 4 }
            0x7A => { self.regs.a = self.regs.d; 4 }
            0x7B => { self.regs.a = self.regs.e; 4 }
            0x7C => { self.regs.a = self.regs.h; 4 }
            0x7D => { self.regs.a = self.regs.l; 4 }
            0x7E => { self.regs.a = mmu.read_byte(self.regs.hl()); 8 }
            0x7F => { 4 } // LD A, A (NOP)
            
            // ========== HALT ==========
            0x76 => {
                self.halted = true;
                4
            }
            
            // ========== ADD A, r8 ==========
            0x80 => { self.add(self.regs.b); 4 }
            0x81 => { self.add(self.regs.c); 4 }
            0x82 => { self.add(self.regs.d); 4 }
            0x83 => { self.add(self.regs.e); 4 }
            0x84 => { self.add(self.regs.h); 4 }
            0x85 => { self.add(self.regs.l); 4 }
            0x86 => { let v = mmu.read_byte(self.regs.hl()); self.add(v); 8 }
            0x87 => { self.add(self.regs.a); 4 }
            
            // ========== ADC A, r8 ==========
            0x88 => { self.adc(self.regs.b); 4 }
            0x89 => { self.adc(self.regs.c); 4 }
            0x8A => { self.adc(self.regs.d); 4 }
            0x8B => { self.adc(self.regs.e); 4 }
            0x8C => { self.adc(self.regs.h); 4 }
            0x8D => { self.adc(self.regs.l); 4 }
            0x8E => { let v = mmu.read_byte(self.regs.hl()); self.adc(v); 8 }
            0x8F => { self.adc(self.regs.a); 4 }
            
            // ========== SUB r8 ==========
            0x90 => { self.sub(self.regs.b); 4 }
            0x91 => { self.sub(self.regs.c); 4 }
            0x92 => { self.sub(self.regs.d); 4 }
            0x93 => { self.sub(self.regs.e); 4 }
            0x94 => { self.sub(self.regs.h); 4 }
            0x95 => { self.sub(self.regs.l); 4 }
            0x96 => { let v = mmu.read_byte(self.regs.hl()); self.sub(v); 8 }
            0x97 => { self.sub(self.regs.a); 4 }
            
            // ========== SBC A, r8 ==========
            0x98 => { self.sbc(self.regs.b); 4 }
            0x99 => { self.sbc(self.regs.c); 4 }
            0x9A => { self.sbc(self.regs.d); 4 }
            0x9B => { self.sbc(self.regs.e); 4 }
            0x9C => { self.sbc(self.regs.h); 4 }
            0x9D => { self.sbc(self.regs.l); 4 }
            0x9E => { let v = mmu.read_byte(self.regs.hl()); self.sbc(v); 8 }
            0x9F => { self.sbc(self.regs.a); 4 }
            
            // ========== AND r8 ==========
            0xA0 => { self.and(self.regs.b); 4 }
            0xA1 => { self.and(self.regs.c); 4 }
            0xA2 => { self.and(self.regs.d); 4 }
            0xA3 => { self.and(self.regs.e); 4 }
            0xA4 => { self.and(self.regs.h); 4 }
            0xA5 => { self.and(self.regs.l); 4 }
            0xA6 => { let v = mmu.read_byte(self.regs.hl()); self.and(v); 8 }
            0xA7 => { self.and(self.regs.a); 4 }
            
            // ========== XOR r8 ==========
            0xA8 => { self.xor(self.regs.b); 4 }
            0xA9 => { self.xor(self.regs.c); 4 }
            0xAA => { self.xor(self.regs.d); 4 }
            0xAB => { self.xor(self.regs.e); 4 }
            0xAC => { self.xor(self.regs.h); 4 }
            0xAD => { self.xor(self.regs.l); 4 }
            0xAE => { let v = mmu.read_byte(self.regs.hl()); self.xor(v); 8 }
            0xAF => { self.xor(self.regs.a); 4 }
            
            // ========== OR r8 ==========
            0xB0 => { self.or(self.regs.b); 4 }
            0xB1 => { self.or(self.regs.c); 4 }
            0xB2 => { self.or(self.regs.d); 4 }
            0xB3 => { self.or(self.regs.e); 4 }
            0xB4 => { self.or(self.regs.h); 4 }
            0xB5 => { self.or(self.regs.l); 4 }
            0xB6 => { let v = mmu.read_byte(self.regs.hl()); self.or(v); 8 }
            0xB7 => { self.or(self.regs.a); 4 }
            
            // ========== CP r8 ==========
            0xB8 => { self.cp(self.regs.b); 4 }
            0xB9 => { self.cp(self.regs.c); 4 }
            0xBA => { self.cp(self.regs.d); 4 }
            0xBB => { self.cp(self.regs.e); 4 }
            0xBC => { self.cp(self.regs.h); 4 }
            0xBD => { self.cp(self.regs.l); 4 }
            0xBE => { let v = mmu.read_byte(self.regs.hl()); self.cp(v); 8 }
            0xBF => { self.cp(self.regs.a); 4 }
            
            // ========== RET cc ==========
            0xC0 => { // RET NZ
                if !self.regs.f.contains(Flags::Z) {
                    self.regs.pc = self.pop_word(mmu);
                    20
                } else {
                    8
                }
            }
            0xC8 => { // RET Z
                if self.regs.f.contains(Flags::Z) {
                    self.regs.pc = self.pop_word(mmu);
                    20
                } else {
                    8
                }
            }
            0xD0 => { // RET NC
                if !self.regs.f.contains(Flags::C) {
                    self.regs.pc = self.pop_word(mmu);
                    20
                } else {
                    8
                }
            }
            0xD8 => { // RET C
                if self.regs.f.contains(Flags::C) {
                    self.regs.pc = self.pop_word(mmu);
                    20
                } else {
                    8
                }
            }
            
            // ========== POP r16 ==========
            0xC1 => { let v = self.pop_word(mmu); self.regs.set_bc(v); 12 }
            0xD1 => { let v = self.pop_word(mmu); self.regs.set_de(v); 12 }
            0xE1 => { let v = self.pop_word(mmu); self.regs.set_hl(v); 12 }
            0xF1 => { let v = self.pop_word(mmu); self.regs.set_af(v); 12 }
            
            // ========== JP cc, nn ==========
            0xC2 => { // JP NZ, nn
                let addr = self.fetch_word(mmu);
                if !self.regs.f.contains(Flags::Z) {
                    self.regs.pc = addr;
                    16
                } else {
                    12
                }
            }
            0xCA => { // JP Z, nn
                let addr = self.fetch_word(mmu);
                if self.regs.f.contains(Flags::Z) {
                    self.regs.pc = addr;
                    16
                } else {
                    12
                }
            }
            0xD2 => { // JP NC, nn
                let addr = self.fetch_word(mmu);
                if !self.regs.f.contains(Flags::C) {
                    self.regs.pc = addr;
                    16
                } else {
                    12
                }
            }
            0xDA => { // JP C, nn
                let addr = self.fetch_word(mmu);
                if self.regs.f.contains(Flags::C) {
                    self.regs.pc = addr;
                    16
                } else {
                    12
                }
            }
            
            // ========== JP nn ==========
            0xC3 => {
                self.regs.pc = self.fetch_word(mmu);
                16
            }
            
            // ========== CALL cc, nn ==========
            0xC4 => { // CALL NZ, nn
                let addr = self.fetch_word(mmu);
                if !self.regs.f.contains(Flags::Z) {
                    self.push_word(mmu, self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else {
                    12
                }
            }
            0xCC => { // CALL Z, nn
                let addr = self.fetch_word(mmu);
                if self.regs.f.contains(Flags::Z) {
                    self.push_word(mmu, self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else {
                    12
                }
            }
            0xD4 => { // CALL NC, nn
                let addr = self.fetch_word(mmu);
                if !self.regs.f.contains(Flags::C) {
                    self.push_word(mmu, self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else {
                    12
                }
            }
            0xDC => { // CALL C, nn
                let addr = self.fetch_word(mmu);
                if self.regs.f.contains(Flags::C) {
                    self.push_word(mmu, self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else {
                    12
                }
            }
            
            // ========== PUSH r16 ==========
            0xC5 => { self.push_word(mmu, self.regs.bc()); 16 }
            0xD5 => { self.push_word(mmu, self.regs.de()); 16 }
            0xE5 => { self.push_word(mmu, self.regs.hl()); 16 }
            0xF5 => { self.push_word(mmu, self.regs.af()); 16 }
            
            // ========== ALU A, n ==========
            0xC6 => { let v = self.fetch_byte(mmu); self.add(v); 8 }
            0xCE => { let v = self.fetch_byte(mmu); self.adc(v); 8 }
            0xD6 => { let v = self.fetch_byte(mmu); self.sub(v); 8 }
            0xDE => { let v = self.fetch_byte(mmu); self.sbc(v); 8 }
            0xE6 => { let v = self.fetch_byte(mmu); self.and(v); 8 }
            0xEE => { let v = self.fetch_byte(mmu); self.xor(v); 8 }
            0xF6 => { let v = self.fetch_byte(mmu); self.or(v); 8 }
            0xFE => { let v = self.fetch_byte(mmu); self.cp(v); 8 }
            
            // ========== RST ==========
            0xC7 => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x00; 16 }
            0xCF => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x08; 16 }
            0xD7 => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x10; 16 }
            0xDF => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x18; 16 }
            0xE7 => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x20; 16 }
            0xEF => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x28; 16 }
            0xF7 => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x30; 16 }
            0xFF => { self.push_word(mmu, self.regs.pc); self.regs.pc = 0x38; 16 }
            
            // ========== RET ==========
            0xC9 => {
                self.regs.pc = self.pop_word(mmu);
                16
            }
            
            // ========== RETI ==========
            0xD9 => {
                self.regs.pc = self.pop_word(mmu);
                self.ime = true;
                16
            }
            
            // ========== CB Prefix ==========
            0xCB => {
                let cb_opcode = self.fetch_byte(mmu);
                self.execute_cb(cb_opcode, mmu)
            }
            
            // ========== CALL nn ==========
            0xCD => {
                let addr = self.fetch_word(mmu);
                self.push_word(mmu, self.regs.pc);
                self.regs.pc = addr;
                24
            }
            
            // ========== LDH (n), A ==========
            0xE0 => {
                let offset = self.fetch_byte(mmu);
                mmu.write_byte(0xFF00 | (offset as u16), self.regs.a);
                12
            }
            
            // ========== LDH (C), A ==========
            0xE2 => {
                mmu.write_byte(0xFF00 | (self.regs.c as u16), self.regs.a);
                8
            }
            
            // ========== JP HL ==========
            0xE9 => {
                self.regs.pc = self.regs.hl();
                4
            }
            
            // ========== LD (nn), A ==========
            0xEA => {
                let addr = self.fetch_word(mmu);
                mmu.write_byte(addr, self.regs.a);
                16
            }
            
            // ========== LDH A, (n) ==========
            0xF0 => {
                let offset = self.fetch_byte(mmu);
                self.regs.a = mmu.read_byte(0xFF00 | (offset as u16));
                12
            }
            
            // ========== LDH A, (C) ==========
            0xF2 => {
                self.regs.a = mmu.read_byte(0xFF00 | (self.regs.c as u16));
                8
            }
            
            // ========== DI ==========
            0xF3 => {
                self.ime = false;
                self.ime_scheduled = false;
                4
            }
            
            // ========== LD SP, HL ==========
            0xF9 => {
                self.regs.sp = self.regs.hl();
                8
            }
            
            // ========== LD A, (nn) ==========
            0xFA => {
                let addr = self.fetch_word(mmu);
                self.regs.a = mmu.read_byte(addr);
                16
            }
            
            // ========== EI ==========
            0xFB => {
                self.ime_scheduled = true;
                4
            }
            
            // ========== ADD SP, e ==========
            0xE8 => {
                let offset = self.fetch_byte(mmu) as i8;
                self.regs.sp = self.add_sp(offset);
                16
            }
            
            // ========== LD HL, SP+e ==========
            0xF8 => {
                let offset = self.fetch_byte(mmu) as i8;
                let result = self.add_sp(offset);
                self.regs.set_hl(result);
                12
            }
            
            // ========== Undefined opcodes ==========
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                // These are undefined and crash on real hardware
                // We'll just treat them as NOPs for now
                4
            }
        }
    }
}
