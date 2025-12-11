//! # PPU (Pixel Processing Unit)
//! 
//! Implements the Game Boy graphics system with accurate timing.
//! 
//! ## Modes
//! - Mode 0: HBlank (204 cycles)
//! - Mode 1: VBlank (4560 cycles)
//! - Mode 2: OAM Search (80 cycles)
//! - Mode 3: Pixel Transfer (172 cycles)

use crate::mmu::Mmu;
use crate::GbModel;
use serde::{Serialize, Deserialize};

/// Screen dimensions
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

/// Framebuffer size (RGBA8888)
pub const FRAMEBUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;

/// Cycles per scanline
const CYCLES_PER_LINE: u32 = 456;

/// Total scanlines (including VBlank)
const TOTAL_LINES: u8 = 154;

/// PPU modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    PixelTransfer = 3,
}

/// PPU step result
pub struct PpuStepResult {
    pub vblank_interrupt: bool,
    pub stat_interrupt: bool,
}

/// Sprite data from OAM
#[derive(Clone, Copy, Default)]
struct Sprite {
    y: u8,
    x: u8,
    tile: u8,
    flags: u8,
}

impl Sprite {
    /// Priority (0 = above BG, 1 = behind BG colors 1-3)
    fn priority(&self) -> bool {
        self.flags & 0x80 != 0
    }
    
    /// Y flip
    fn y_flip(&self) -> bool {
        self.flags & 0x40 != 0
    }
    
    /// X flip
    fn x_flip(&self) -> bool {
        self.flags & 0x20 != 0
    }
    
    /// Palette (DMG: OBP0/OBP1, CGB: palette number)
    fn palette(&self) -> u8 {
        if self.flags & 0x10 != 0 { 1 } else { 0 }
    }
    
    /// VRAM bank (CGB only)
    #[allow(dead_code)]
    fn vram_bank(&self) -> u8 {
        if self.flags & 0x08 != 0 { 1 } else { 0 }
    }
    
    /// CGB palette number
    #[allow(dead_code)]
    fn cgb_palette(&self) -> u8 {
        self.flags & 0x07
    }
}

/// PPU state for serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct PpuState {
    pub mode: PpuMode,
    pub cycles: u32,
    pub ly: u8,
    pub window_line: u8,
    pub stat_interrupt_line: bool,
    pub bg_palette: [[u8; 4]; 8],
    pub obj_palette: [[u8; 4]; 8],
}

/// Pixel Processing Unit
pub struct Ppu {
    /// Current mode
    mode: PpuMode,
    
    /// Cycles in current mode
    cycles: u32,
    
    /// Current scanline (LY)
    ly: u8,
    
    /// Window internal line counter
    window_line: u8,
    
    /// Framebuffer (RGBA8888)
    framebuffer: Vec<u8>,
    
    /// Game Boy model
    model: GbModel,
    
    /// STAT interrupt line (for edge detection)
    stat_interrupt_line: bool,
    
    /// CGB background palettes (8 palettes, 4 colors each, RGB555)
    bg_palette: [[u8; 4]; 8],
    
    /// CGB object palettes
    obj_palette: [[u8; 4]; 8],
    
    /// CGB background palette data (for future CGB support)
    #[allow(dead_code)]
    bg_palette_data: [u8; 64],
    
    /// CGB object palette data (for future CGB support)
    #[allow(dead_code)]
    obj_palette_data: [u8; 64],
}

impl Ppu {
    /// Create a new PPU
    pub fn new(model: GbModel) -> Self {
        Self {
            mode: PpuMode::OamSearch,
            cycles: 0,
            ly: 0,
            window_line: 0,
            framebuffer: vec![0xFF; FRAMEBUFFER_SIZE],
            model,
            stat_interrupt_line: false,
            bg_palette: [[0; 4]; 8],
            obj_palette: [[0; 4]; 8],
            bg_palette_data: [0xFF; 64],
            obj_palette_data: [0xFF; 64],
        }
    }
    
    /// Reset PPU
    pub fn reset(&mut self) {
        self.mode = PpuMode::OamSearch;
        self.cycles = 0;
        self.ly = 0;
        self.window_line = 0;
        self.framebuffer.fill(0xFF);
        self.stat_interrupt_line = false;
    }
    
    /// Step the PPU
    pub fn step(&mut self, cycles: u32, mmu: &mut Mmu) -> PpuStepResult {
        let mut result = PpuStepResult {
            vblank_interrupt: false,
            stat_interrupt: false,
        };
        
        let lcdc = mmu.io()[0x40];
        
        // LCD disabled
        if lcdc & 0x80 == 0 {
            self.mode = PpuMode::HBlank;
            self.ly = 0;
            self.cycles = 0;
            mmu.io_mut()[0x44] = 0;
            mmu.io_mut()[0x41] &= 0xFC;
            return result;
        }
        
        self.cycles += cycles;
        
        // Process mode transitions
        match self.mode {
            PpuMode::OamSearch => {
                if self.cycles >= 80 {
                    self.cycles -= 80;
                    self.mode = PpuMode::PixelTransfer;
                }
            }
            
            PpuMode::PixelTransfer => {
                if self.cycles >= 172 {
                    self.cycles -= 172;
                    self.mode = PpuMode::HBlank;
                    
                    // Render scanline
                    if self.ly < SCREEN_HEIGHT as u8 {
                        self.render_scanline(mmu);
                    }
                    
                    // HBlank STAT interrupt
                    let stat = mmu.io()[0x41];
                    if stat & 0x08 != 0 {
                        result.stat_interrupt = self.check_stat_interrupt(mmu);
                    }
                    
                    // HBlank HDMA (CGB)
                    mmu.step_hblank_hdma();
                }
            }
            
            PpuMode::HBlank => {
                if self.cycles >= 204 {
                    self.cycles -= 204;
                    self.ly += 1;
                    mmu.io_mut()[0x44] = self.ly;
                    
                    if self.ly == 144 {
                        self.mode = PpuMode::VBlank;
                        result.vblank_interrupt = true;
                        self.window_line = 0;
                        
                        // VBlank STAT interrupt
                        let stat = mmu.io()[0x41];
                        if stat & 0x10 != 0 {
                            result.stat_interrupt = self.check_stat_interrupt(mmu);
                        }
                    } else {
                        self.mode = PpuMode::OamSearch;
                        
                        // OAM STAT interrupt
                        let stat = mmu.io()[0x41];
                        if stat & 0x20 != 0 {
                            result.stat_interrupt = self.check_stat_interrupt(mmu);
                        }
                    }
                    
                    // LYC=LY check
                    self.check_lyc(mmu, &mut result);
                }
            }
            
            PpuMode::VBlank => {
                if self.cycles >= CYCLES_PER_LINE {
                    self.cycles -= CYCLES_PER_LINE;
                    self.ly += 1;
                    
                    if self.ly >= TOTAL_LINES {
                        self.ly = 0;
                        self.mode = PpuMode::OamSearch;
                        
                        // OAM STAT interrupt
                        let stat = mmu.io()[0x41];
                        if stat & 0x20 != 0 {
                            result.stat_interrupt = self.check_stat_interrupt(mmu);
                        }
                    }
                    
                    mmu.io_mut()[0x44] = self.ly;
                    self.check_lyc(mmu, &mut result);
                }
            }
        }
        
        // Update STAT mode bits
        let stat = mmu.io()[0x41];
        mmu.io_mut()[0x41] = (stat & 0xFC) | (self.mode as u8);
        
        result
    }
    
    /// Check LYC=LY and trigger STAT interrupt if needed
    fn check_lyc(&mut self, mmu: &mut Mmu, result: &mut PpuStepResult) {
        let lyc = mmu.io()[0x45];
        let stat = mmu.io()[0x41];
        
        if self.ly == lyc {
            // Set coincidence flag
            mmu.io_mut()[0x41] = stat | 0x04;
            
            // LYC=LY STAT interrupt
            if stat & 0x40 != 0 {
                result.stat_interrupt = self.check_stat_interrupt(mmu);
            }
        } else {
            // Clear coincidence flag
            mmu.io_mut()[0x41] = stat & !0x04;
        }
    }
    
    /// Check STAT interrupt with edge detection
    fn check_stat_interrupt(&mut self, _mmu: &Mmu) -> bool {
        let was_high = self.stat_interrupt_line;
        self.stat_interrupt_line = true;
        !was_high
    }
    
    /// Render a single scanline
    fn render_scanline(&mut self, mmu: &Mmu) {
        let lcdc = mmu.io()[0x40];
        let ly = self.ly;
        
        // Clear scanline to white
        let offset = ly as usize * SCREEN_WIDTH * 4;
        for x in 0..SCREEN_WIDTH {
            let i = offset + x * 4;
            self.framebuffer[i] = 0xFF;
            self.framebuffer[i + 1] = 0xFF;
            self.framebuffer[i + 2] = 0xFF;
            self.framebuffer[i + 3] = 0xFF;
        }
        
        // Background priority array (for sprite rendering)
        let mut bg_priority = [0u8; SCREEN_WIDTH];
        
        // Render background
        if lcdc & 0x01 != 0 || matches!(self.model, GbModel::Cgb | GbModel::CgbDmg) {
            self.render_background(mmu, &mut bg_priority);
        }
        
        // Render window
        if lcdc & 0x20 != 0 {
            self.render_window(mmu, &mut bg_priority);
        }
        
        // Render sprites
        if lcdc & 0x02 != 0 {
            self.render_sprites(mmu, &bg_priority);
        }
    }
    
    /// Render background for current scanline
    fn render_background(&mut self, mmu: &Mmu, bg_priority: &mut [u8; SCREEN_WIDTH]) {
        let lcdc = mmu.io()[0x40];
        let scx = mmu.io()[0x43];
        let scy = mmu.io()[0x42];
        let bgp = mmu.io()[0x47];
        
        let tile_map_base: u16 = if lcdc & 0x08 != 0 { 0x9C00 } else { 0x9800 };
        let signed_addressing = lcdc & 0x10 == 0;
        
        let y = self.ly.wrapping_add(scy);
        let tile_row = (y / 8) as u16;
        let pixel_row = (y % 8) as u16;
        
        for screen_x in 0..SCREEN_WIDTH {
            let x = (screen_x as u8).wrapping_add(scx);
            let tile_col = (x / 8) as u16;
            let pixel_col = 7 - (x % 8);
            
            // Get tile index from tile map
            let map_addr = tile_map_base + (tile_row * 32) + tile_col;
            let tile_index = mmu.read_byte(map_addr);
            
            // Calculate tile data address
            let tile_addr = if signed_addressing {
                // Base is 0x9000, tile index is signed (-128 to 127)
                let signed_index = tile_index as i8 as i16;
                (0x9000i32 + (signed_index as i32 * 16) + (pixel_row as i32 * 2)) as u16
            } else {
                // Base is 0x8000, tile index is unsigned (0 to 255)
                0x8000 + (tile_index as u16 * 16) + (pixel_row * 2)
            };
            
            // Get tile data
            let low = mmu.read_byte(tile_addr);
            let high = mmu.read_byte(tile_addr.wrapping_add(1));
            
            // Get color index
            let color_index = ((high >> pixel_col) & 1) << 1 | ((low >> pixel_col) & 1);
            
            bg_priority[screen_x] = color_index;
            
            // Apply palette and draw pixel
            let color = self.apply_dmg_palette(color_index, bgp);
            self.set_pixel(screen_x, self.ly as usize, color);
        }
    }
    
    /// Render window for current scanline
    fn render_window(&mut self, mmu: &Mmu, bg_priority: &mut [u8; SCREEN_WIDTH]) {
        let lcdc = mmu.io()[0x40];
        let wy = mmu.io()[0x4A];
        let wx = mmu.io()[0x4B];
        let bgp = mmu.io()[0x47];
        
        // Window not visible on this line
        if self.ly < wy || wx > 166 {
            return;
        }
        
        let tile_map_base: u16 = if lcdc & 0x40 != 0 { 0x9C00 } else { 0x9800 };
        let signed_addressing = lcdc & 0x10 == 0;
        
        let window_y = self.window_line;
        let tile_row = (window_y / 8) as u16;
        let pixel_row = (window_y % 8) as u16;
        
        let window_x_start = wx.saturating_sub(7) as usize;
        let mut drew_window = false;
        
        for screen_x in window_x_start..SCREEN_WIDTH {
            let window_x = (screen_x - window_x_start) as u8;
            let tile_col = (window_x / 8) as u16;
            let pixel_col = 7 - (window_x % 8);
            
            let map_addr = tile_map_base + (tile_row * 32) + tile_col;
            let tile_index = mmu.read_byte(map_addr);
            
            let tile_addr = if signed_addressing {
                // Base is 0x9000, tile index is signed (-128 to 127)
                let signed_index = tile_index as i8 as i16;
                (0x9000i32 + (signed_index as i32 * 16) + (pixel_row as i32 * 2)) as u16
            } else {
                // Base is 0x8000, tile index is unsigned (0 to 255)
                0x8000 + (tile_index as u16 * 16) + (pixel_row * 2)
            };
            
            let low = mmu.read_byte(tile_addr);
            let high = mmu.read_byte(tile_addr.wrapping_add(1));
            
            let color_index = ((high >> pixel_col) & 1) << 1 | ((low >> pixel_col) & 1);
            
            bg_priority[screen_x] = color_index;
            
            let color = self.apply_dmg_palette(color_index, bgp);
            self.set_pixel(screen_x, self.ly as usize, color);
            
            drew_window = true;
        }
        
        if drew_window {
            self.window_line += 1;
        }
    }
    
    /// Render sprites for current scanline
    fn render_sprites(&mut self, mmu: &Mmu, bg_priority: &[u8; SCREEN_WIDTH]) {
        let lcdc = mmu.io()[0x40];
        let obp0 = mmu.io()[0x48];
        let obp1 = mmu.io()[0x49];
        
        let sprite_height: i32 = if lcdc & 0x04 != 0 { 16 } else { 8 };
        let oam = mmu.oam();
        
        // Collect sprites on this scanline (max 10)
        let mut sprites: Vec<(usize, Sprite)> = Vec::with_capacity(10);
        
        let ly = self.ly as i32;
        
        for i in 0..40 {
            let offset = i * 4;
            let sprite = Sprite {
                y: oam[offset],
                x: oam[offset + 1],
                tile: oam[offset + 2],
                flags: oam[offset + 3],
            };
            
            // Sprite Y is offset by 16 (sprite.y = 16 means top of sprite at screen Y=0)
            let sprite_y = sprite.y as i32 - 16;
            
            // Check if sprite is on this scanline
            if ly >= sprite_y && ly < sprite_y + sprite_height {
                sprites.push((i, sprite));
                if sprites.len() >= 10 {
                    break;
                }
            }
        }
        
        // Sort by X coordinate (lower X = higher priority)
        // For DMG, on equal X, lower OAM index wins
        sprites.sort_by(|a, b| {
            if a.1.x == b.1.x {
                a.0.cmp(&b.0)
            } else {
                a.1.x.cmp(&b.1.x)
            }
        });
        
        // Render sprites in reverse order (so higher priority draws last)
        for (_, sprite) in sprites.iter().rev() {
            let sprite_x = sprite.x as i32 - 8;
            let sprite_y = sprite.y as i32 - 16;
            
            // Calculate which row of the sprite to draw
            let mut row = (ly - sprite_y) as u8;
            if sprite.y_flip() {
                row = (sprite_height as u8) - 1 - row;
            }
            
            // For 8x16 sprites, select the correct tile
            let tile = if sprite_height == 16 {
                if row >= 8 {
                    sprite.tile | 0x01
                } else {
                    sprite.tile & 0xFE
                }
            } else {
                sprite.tile
            };
            
            let row = row % 8;
            
            // Get tile data (sprites always use 0x8000 addressing)
            let tile_addr = 0x8000 + (tile as u16 * 16) + (row as u16 * 2);
            let low = mmu.read_byte(tile_addr);
            let high = mmu.read_byte(tile_addr + 1);
            
            // Draw each pixel of the sprite
            for pixel_x in 0..8i32 {
                let screen_x = sprite_x + pixel_x;
                
                if screen_x < 0 || screen_x >= SCREEN_WIDTH as i32 {
                    continue;
                }
                
                let screen_x = screen_x as usize;
                
                // Apply X flip
                let bit = if sprite.x_flip() {
                    pixel_x as u8
                } else {
                    7 - pixel_x as u8
                };
                
                let color_index = ((high >> bit) & 1) << 1 | ((low >> bit) & 1);
                
                // Color 0 is transparent for sprites
                if color_index == 0 {
                    continue;
                }
                
                // Check BG priority
                // If sprite has BG priority flag set AND bg pixel is not color 0, skip
                if sprite.priority() && bg_priority[screen_x] != 0 {
                    continue;
                }
                
                // Apply palette
                let palette = if sprite.palette() == 0 { obp0 } else { obp1 };
                let color = self.apply_dmg_palette(color_index, palette);
                
                self.set_pixel(screen_x, self.ly as usize, color);
            }
        }
    }
    
    /// Apply DMG palette to color index
    fn apply_dmg_palette(&self, color_index: u8, palette: u8) -> [u8; 4] {
        let shade = (palette >> (color_index * 2)) & 0x03;
        
        // Warm beige/sepia tones - easy on the eyes
        match shade {
            0 => [0xF5, 0xF0, 0xE6, 0xFF], // Lightest - warm white/cream
            1 => [0xC8, 0xB8, 0x9A, 0xFF], // Light beige
            2 => [0x7A, 0x6A, 0x52, 0xFF], // Dark brown
            3 => [0x26, 0x22, 0x1C, 0xFF], // Darkest - near black with warm tint
            _ => unreachable!(),
        }
    }
    
    /// Set pixel in framebuffer
    fn set_pixel(&mut self, x: usize, y: usize, color: [u8; 4]) {
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            let offset = (y * SCREEN_WIDTH + x) * 4;
            self.framebuffer[offset..offset + 4].copy_from_slice(&color);
        }
    }
    
    /// Get framebuffer
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }
    
    /// Get current state for serialization
    pub fn state(&self) -> PpuState {
        PpuState {
            mode: self.mode,
            cycles: self.cycles,
            ly: self.ly,
            window_line: self.window_line,
            stat_interrupt_line: self.stat_interrupt_line,
            bg_palette: self.bg_palette,
            obj_palette: self.obj_palette,
        }
    }
    
    /// Load state from serialization
    pub fn load_state(&mut self, state: PpuState) {
        self.mode = state.mode;
        self.cycles = state.cycles;
        self.ly = state.ly;
        self.window_line = state.window_line;
        self.stat_interrupt_line = state.stat_interrupt_line;
        self.bg_palette = state.bg_palette;
        self.obj_palette = state.obj_palette;
    }
}