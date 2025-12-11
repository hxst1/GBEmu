//! # Serial Module
//! 
//! Handles serial communication (Link Cable).
//! For now, this is a minimal implementation that just handles
//! the timing for internal clock mode.

/// Serial port implementation
pub struct Serial {
    /// Serial transfer data
    data: u8,
    
    /// Serial control
    control: u8,
    
    /// Transfer counter
    transfer_counter: u32,
    
    /// Bits remaining to transfer
    bits_remaining: u8,
}

impl Serial {
    pub fn new() -> Self {
        Self {
            data: 0,
            control: 0,
            transfer_counter: 0,
            bits_remaining: 0,
        }
    }
    
    pub fn reset(&mut self) {
        self.data = 0;
        self.control = 0;
        self.transfer_counter = 0;
        self.bits_remaining = 0;
    }
    
    /// Step serial transfer
    /// Returns true if serial interrupt should be requested
    pub fn step(&mut self, cycles: u32) -> bool {
        // Check if transfer is active with internal clock
        if self.control & 0x81 != 0x81 {
            return false;
        }
        
        self.transfer_counter += cycles;
        
        // Transfer at 8192 Hz (512 cycles per bit)
        while self.transfer_counter >= 512 && self.bits_remaining > 0 {
            self.transfer_counter -= 512;
            self.bits_remaining -= 1;
            
            // Shift in 1 (no external device connected)
            self.data = (self.data << 1) | 1;
            
            if self.bits_remaining == 0 {
                // Transfer complete
                self.control &= !0x80;
                return true;
            }
        }
        
        false
    }
    
    /// Read serial data register
    pub fn read_data(&self) -> u8 {
        self.data
    }
    
    /// Write serial data register
    pub fn write_data(&mut self, value: u8) {
        self.data = value;
    }
    
    /// Read serial control register
    pub fn read_control(&self) -> u8 {
        self.control | 0x7E
    }
    
    /// Write serial control register
    pub fn write_control(&mut self, value: u8) {
        self.control = value;
        
        // Start transfer if bit 7 is set
        if value & 0x80 != 0 {
            self.bits_remaining = 8;
            self.transfer_counter = 0;
        }
    }
}
