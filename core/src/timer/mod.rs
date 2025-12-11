//! # Timer Module
//! 
//! Implements the Game Boy timer system:
//! - DIV: Divider register (increments at 16384 Hz)
//! - TIMA: Timer counter
//! - TMA: Timer modulo
//! - TAC: Timer control

use serde::{Serialize, Deserialize};

/// Timer state for serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub div_counter: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub tima_overflow: bool,
    pub tima_reload_cycle: bool,
}

/// Timer implementation
pub struct Timer {
    /// Internal DIV counter (16-bit, upper 8 bits are DIV register)
    div_counter: u16,
    
    /// Timer counter
    tima: u8,
    
    /// Timer modulo
    tma: u8,
    
    /// Timer control
    tac: u8,
    
    /// TIMA overflow happened (delay interrupt by 1 cycle)
    tima_overflow: bool,
    
    /// TIMA reload cycle
    tima_reload_cycle: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_counter: 0xABCC, // Post-boot value
            tima: 0,
            tma: 0,
            tac: 0,
            tima_overflow: false,
            tima_reload_cycle: false,
        }
    }
    
    pub fn reset(&mut self) {
        self.div_counter = 0;
        self.tima = 0;
        self.tma = 0;
        self.tac = 0;
        self.tima_overflow = false;
        self.tima_reload_cycle = false;
    }
    
    /// Step the timer by CPU cycles
    /// Returns true if timer interrupt should be requested
    pub fn step(&mut self, cycles: u32) -> bool {
        let mut interrupt = false;
        
        for _ in 0..cycles {
            // Check for reload cycle first
            if self.tima_reload_cycle {
                self.tima_reload_cycle = false;
                self.tima = self.tma;
                interrupt = true;
            }
            
            // Check overflow from previous cycle
            if self.tima_overflow {
                self.tima_overflow = false;
                self.tima_reload_cycle = true;
            }
            
            // Get the bit position to check based on TAC
            let old_div = self.div_counter;
            self.div_counter = self.div_counter.wrapping_add(1);
            
            // Check if timer is enabled
            if self.tac & 0x04 != 0 {
                let bit_pos = match self.tac & 0x03 {
                    0 => 9,  // 4096 Hz
                    1 => 3,  // 262144 Hz
                    2 => 5,  // 65536 Hz
                    3 => 7,  // 16384 Hz
                    _ => unreachable!(),
                };
                
                // Falling edge detection
                let old_bit = (old_div >> bit_pos) & 1;
                let new_bit = (self.div_counter >> bit_pos) & 1;
                
                if old_bit == 1 && new_bit == 0 {
                    self.tima = self.tima.wrapping_add(1);
                    if self.tima == 0 {
                        self.tima_overflow = true;
                    }
                }
            }
        }
        
        interrupt
    }
    
    /// Read DIV register
    pub fn read_div(&self) -> u8 {
        (self.div_counter >> 8) as u8
    }
    
    /// Write DIV register (resets to 0)
    pub fn write_div(&mut self) {
        // Writing any value resets the entire counter
        // This can cause a TIMA increment if the selected bit was 1
        let bit_pos = match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        };
        
        if self.tac & 0x04 != 0 && (self.div_counter >> bit_pos) & 1 == 1 {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.tima_overflow = true;
            }
        }
        
        self.div_counter = 0;
    }
    
    /// Read TIMA register
    pub fn read_tima(&self) -> u8 {
        self.tima
    }
    
    /// Write TIMA register
    pub fn write_tima(&mut self, value: u8) {
        // Writing during reload cycle is ignored
        if !self.tima_reload_cycle {
            self.tima = value;
            // Cancel pending overflow
            self.tima_overflow = false;
        }
    }
    
    /// Read TMA register
    pub fn read_tma(&self) -> u8 {
        self.tma
    }
    
    /// Write TMA register
    pub fn write_tma(&mut self, value: u8) {
        self.tma = value;
        // If written during reload cycle, TIMA gets the new value too
        if self.tima_reload_cycle {
            self.tima = value;
        }
    }
    
    /// Read TAC register
    pub fn read_tac(&self) -> u8 {
        self.tac | 0xF8
    }
    
    /// Write TAC register
    pub fn write_tac(&mut self, value: u8) {
        let old_enabled = self.tac & 0x04 != 0;
        let old_bit_pos = match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        };
        
        self.tac = value & 0x07;
        
        let new_enabled = self.tac & 0x04 != 0;
        let new_bit_pos = match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        };
        
        // Glitch: changing TAC can cause TIMA increment
        let old_bit = if old_enabled { (self.div_counter >> old_bit_pos) & 1 } else { 0 };
        let new_bit = if new_enabled { (self.div_counter >> new_bit_pos) & 1 } else { 0 };
        
        if old_bit == 1 && new_bit == 0 {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.tima_overflow = true;
            }
        }
    }
    
    /// Get current state for serialization
    pub fn state(&self) -> TimerState {
        TimerState {
            div_counter: self.div_counter,
            tima: self.tima,
            tma: self.tma,
            tac: self.tac,
            tima_overflow: self.tima_overflow,
            tima_reload_cycle: self.tima_reload_cycle,
        }
    }
    
    /// Load state from serialization
    pub fn load_state(&mut self, state: TimerState) {
        self.div_counter = state.div_counter;
        self.tima = state.tima;
        self.tma = state.tma;
        self.tac = state.tac;
        self.tima_overflow = state.tima_overflow;
        self.tima_reload_cycle = state.tima_reload_cycle;
    }
}
