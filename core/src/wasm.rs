//! # WASM Bindings
//! 
//! Exposes the emulator core to JavaScript/TypeScript via wasm-bindgen.

use wasm_bindgen::prelude::*;
use crate::{GameBoy, Button};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// WASM-exposed Game Boy emulator
#[wasm_bindgen]
pub struct WasmGameBoy {
    inner: GameBoy,
}

#[wasm_bindgen]
impl WasmGameBoy {
    /// Create a new Game Boy emulator instance
    #[wasm_bindgen(constructor)]
    pub fn new(rom_data: &[u8]) -> Result<WasmGameBoy, JsValue> {
        let gb = GameBoy::new(rom_data)
            .map_err(|e| JsValue::from_str(&e))?;
        
        Ok(WasmGameBoy { inner: gb })
    }
    
    /// Reset the emulator
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    
    /// Run one frame and return pointer to framebuffer
    /// The framebuffer is RGBA8888 format, 160x144 pixels
    #[wasm_bindgen]
    pub fn run_frame(&mut self) -> *const u8 {
        self.inner.run_frame().as_ptr()
    }
    
    /// Get framebuffer as a copy (for safer JS access)
    #[wasm_bindgen]
    pub fn get_framebuffer(&self) -> Vec<u8> {
        self.inner.framebuffer().to_vec()
    }
    
    /// Get framebuffer width
    #[wasm_bindgen]
    pub fn screen_width(&self) -> u32 {
        crate::ppu::SCREEN_WIDTH as u32
    }
    
    /// Get framebuffer height
    #[wasm_bindgen]
    pub fn screen_height(&self) -> u32 {
        crate::ppu::SCREEN_HEIGHT as u32
    }
    
    /// Run for a specific number of cycles
    #[wasm_bindgen]
    pub fn run_cycles(&mut self, cycles: u32) {
        self.inner.run_cycles(cycles);
    }
    
    /// Press a button
    /// Button codes: 0=Right, 1=Left, 2=Up, 3=Down, 4=A, 5=B, 6=Select, 7=Start
    #[wasm_bindgen]
    pub fn press_button(&mut self, code: u8) {
        if let Some(button) = Button::from_code(code) {
            self.inner.press_button(button);
        }
    }
    
    /// Release a button
    #[wasm_bindgen]
    pub fn release_button(&mut self, code: u8) {
        if let Some(button) = Button::from_code(code) {
            self.inner.release_button(button);
        }
    }
    
    /// Save SRAM (battery-backed save data)
    #[wasm_bindgen]
    pub fn save_sram(&self) -> Option<Vec<u8>> {
        self.inner.save_sram()
    }
    
    /// Load SRAM
    #[wasm_bindgen]
    pub fn load_sram(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.inner.load_sram(data)
            .map_err(|e| JsValue::from_str(&e))
    }
    
    /// Create a save state
    #[wasm_bindgen]
    pub fn save_state(&self) -> Vec<u8> {
        self.inner.save_state()
    }
    
    /// Load a save state
    #[wasm_bindgen]
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.inner.load_state(data)
            .map_err(|e| JsValue::from_str(&e))
    }
    
    /// Get game title
    #[wasm_bindgen]
    pub fn game_title(&self) -> String {
        self.inner.game_title().to_string()
    }
    
    /// Check if this is a CGB game
    #[wasm_bindgen]
    pub fn is_cgb_game(&self) -> bool {
        self.inner.is_cgb_game()
    }
    
    /// Get audio samples (stereo interleaved)
    #[wasm_bindgen]
    pub fn get_audio_buffer(&self) -> Vec<f32> {
        self.inner.audio_buffer().to_vec()
    }
    
    /// Clear audio buffer after reading
    #[wasm_bindgen]
    pub fn clear_audio_buffer(&mut self) {
        self.inner.clear_audio_buffer();
    }
    
    /// Get audio sample rate
    #[wasm_bindgen]
    pub fn audio_sample_rate(&self) -> u32 {
        crate::apu::SAMPLE_RATE
    }
    
    /// Get frame count
    #[wasm_bindgen]
    pub fn frame_count(&self) -> u64 {
        self.inner.frame_count()
    }
    
    /// Get total cycles executed
    #[wasm_bindgen]
    pub fn total_cycles(&self) -> u64 {
        self.inner.total_cycles()
    }
}

// Button constants exported individually
#[wasm_bindgen]
pub fn button_right() -> u8 { 0 }
#[wasm_bindgen]
pub fn button_left() -> u8 { 1 }
#[wasm_bindgen]
pub fn button_up() -> u8 { 2 }
#[wasm_bindgen]
pub fn button_down() -> u8 { 3 }
#[wasm_bindgen]
pub fn button_a() -> u8 { 4 }
#[wasm_bindgen]
pub fn button_b() -> u8 { 5 }
#[wasm_bindgen]
pub fn button_select() -> u8 { 6 }
#[wasm_bindgen]
pub fn button_start() -> u8 { 7 }
