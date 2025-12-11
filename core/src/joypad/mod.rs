//! # Joypad Module
//! 
//! Handles button input for the Game Boy.
//! 
//! ## Button Layout
//! - D-pad: Up, Down, Left, Right
//! - Action: A, B, Start, Select

use serde::{Serialize, Deserialize};

/// Button codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Right = 0,
    Left = 1,
    Up = 2,
    Down = 3,
    A = 4,
    B = 5,
    Select = 6,
    Start = 7,
}

impl Button {
    /// Create button from numeric code
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Button::Right),
            1 => Some(Button::Left),
            2 => Some(Button::Up),
            3 => Some(Button::Down),
            4 => Some(Button::A),
            5 => Some(Button::B),
            6 => Some(Button::Select),
            7 => Some(Button::Start),
            _ => None,
        }
    }
}

/// Joypad state for serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct JoypadState {
    pub buttons: u8,
    pub interrupt_pending: bool,
}

/// Joypad implementation
pub struct Joypad {
    /// Button state (bit = 0 means pressed)
    /// Bits 0-3: Right, Left, Up, Down
    /// Bits 4-7: A, B, Select, Start
    buttons: u8,
    
    /// Interrupt pending flag
    interrupt_pending: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            buttons: 0xFF, // All buttons released
            interrupt_pending: false,
        }
    }
    
    pub fn reset(&mut self) {
        self.buttons = 0xFF;
        self.interrupt_pending = false;
    }
    
    /// Press a button
    pub fn press(&mut self, button: Button) {
        let old_buttons = self.buttons;
        self.buttons &= !(1 << (button as u8));
        
        // Trigger interrupt on button press
        if old_buttons != self.buttons {
            self.interrupt_pending = true;
        }
    }
    
    /// Release a button
    pub fn release(&mut self, button: Button) {
        self.buttons |= 1 << (button as u8);
    }
    
    /// Check if a button is pressed
    pub fn is_pressed(&self, button: Button) -> bool {
        self.buttons & (1 << (button as u8)) == 0
    }
    
    /// Read joypad register based on selection
    pub fn read(&self, select: u8) -> u8 {
        let mut result = select | 0xC0; // Bits 6-7 always 1
        
        // Select buttons (bit 5 = 0)
        if select & 0x20 == 0 {
            result &= 0xF0 | ((self.buttons >> 4) & 0x0F);
        }
        
        // Select d-pad (bit 4 = 0)
        if select & 0x10 == 0 {
            result &= 0xF0 | (self.buttons & 0x0F);
        }
        
        result
    }
    
    /// Check and clear interrupt flag
    pub fn check_interrupt(&mut self) -> bool {
        let pending = self.interrupt_pending;
        self.interrupt_pending = false;
        pending
    }
    
    /// Get raw button state (for MMU)
    /// Bits 0-3: Right, Left, Up, Down
    /// Bits 4-7: A, B, Select, Start
    /// Bit = 0 means pressed
    pub fn buttons(&self) -> u8 {
        self.buttons
    }
    
    /// Get current state for serialization
    pub fn state(&self) -> JoypadState {
        JoypadState {
            buttons: self.buttons,
            interrupt_pending: self.interrupt_pending,
        }
    }
    
    /// Load state from serialization
    pub fn load_state(&mut self, state: JoypadState) {
        self.buttons = state.buttons;
        self.interrupt_pending = state.interrupt_pending;
    }
}