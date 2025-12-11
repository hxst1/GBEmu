//! # APU (Audio Processing Unit)
//! 
//! Implements the Game Boy sound system with 4 channels:
//! - Channel 1: Square wave with sweep
//! - Channel 2: Square wave
//! - Channel 3: Wave output
//! - Channel 4: Noise

use serde::{Serialize, Deserialize};

/// Audio sample rate
pub const SAMPLE_RATE: u32 = 44100;

/// CPU cycles per audio sample
const CYCLES_PER_SAMPLE: u32 = 4_194_304 / SAMPLE_RATE;

/// Frame sequencer rate (512 Hz)
const FRAME_SEQUENCER_RATE: u32 = 4_194_304 / 512;

/// APU state for serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct ApuState {
    pub enabled: bool,
    pub frame_sequencer_step: u8,
    pub channel1: Channel1State,
    pub channel2: Channel2State,
    pub channel3: Channel3State,
    pub channel4: Channel4State,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Channel1State {
    pub enabled: bool,
    pub dac_enabled: bool,
    pub length_counter: u8,
    pub frequency: u16,
    pub duty: u8,
    pub volume: u8,
    pub envelope_timer: u8,
    pub envelope_direction: bool,
    pub envelope_period: u8,
    pub sweep_timer: u8,
    pub sweep_period: u8,
    pub sweep_direction: bool,
    pub sweep_shift: u8,
    pub sweep_enabled: bool,
    pub shadow_frequency: u16,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Channel2State {
    pub enabled: bool,
    pub dac_enabled: bool,
    pub length_counter: u8,
    pub frequency: u16,
    pub duty: u8,
    pub volume: u8,
    pub envelope_timer: u8,
    pub envelope_direction: bool,
    pub envelope_period: u8,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Channel3State {
    pub enabled: bool,
    pub dac_enabled: bool,
    pub length_counter: u16,
    pub frequency: u16,
    pub volume_code: u8,
    pub sample_index: u8,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Channel4State {
    pub enabled: bool,
    pub dac_enabled: bool,
    pub length_counter: u8,
    pub volume: u8,
    pub envelope_timer: u8,
    pub envelope_direction: bool,
    pub envelope_period: u8,
    pub lfsr: u16,
    pub clock_shift: u8,
    pub width_mode: bool,
    pub divisor_code: u8,
}

/// Square wave channel with sweep (Channel 1)
struct Channel1 {
    enabled: bool,
    dac_enabled: bool,
    
    // Length counter
    length_counter: u8,
    length_enabled: bool,
    
    // Frequency
    frequency: u16,
    frequency_timer: u32,
    duty_position: u8,
    duty: u8,
    
    // Volume envelope
    volume: u8,
    initial_volume: u8,
    envelope_timer: u8,
    envelope_direction: bool,
    envelope_period: u8,
    
    // Sweep
    sweep_timer: u8,
    sweep_period: u8,
    sweep_direction: bool,
    sweep_shift: u8,
    sweep_enabled: bool,
    shadow_frequency: u16,
}

impl Default for Channel1 {
    fn default() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            length_counter: 0,
            length_enabled: false,
            frequency: 0,
            frequency_timer: 0,
            duty_position: 0,
            duty: 0,
            volume: 0,
            initial_volume: 0,
            envelope_timer: 0,
            envelope_direction: false,
            envelope_period: 0,
            sweep_timer: 0,
            sweep_period: 0,
            sweep_direction: false,
            sweep_shift: 0,
            sweep_enabled: false,
            shadow_frequency: 0,
        }
    }
}

impl Channel1 {
    fn step(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }
        
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency as u32) * 4;
            self.duty_position = (self.duty_position + 1) & 7;
        }
    }
    
    fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }
        
        let duty_table: [[u8; 8]; 4] = [
            [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
            [1, 0, 0, 0, 0, 0, 0, 1], // 25%
            [1, 0, 0, 0, 0, 1, 1, 1], // 50%
            [0, 1, 1, 1, 1, 1, 1, 0], // 75%
        ];
        
        let sample = duty_table[self.duty as usize][self.duty_position as usize];
        let volume = self.volume as f32 / 15.0;
        
        if sample == 1 { volume } else { -volume }
    }
    
    fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
    
    fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
            return;
        }
        
        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }
        
        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_period;
            
            if self.envelope_direction && self.volume < 15 {
                self.volume += 1;
            } else if !self.envelope_direction && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
    
    fn clock_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }
        
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };
            
            if self.sweep_enabled && self.sweep_period > 0 {
                let new_freq = self.calculate_sweep_frequency();
                
                if new_freq <= 2047 && self.sweep_shift > 0 {
                    self.frequency = new_freq;
                    self.shadow_frequency = new_freq;
                    
                    // Overflow check
                    self.calculate_sweep_frequency();
                }
            }
        }
    }
    
    fn calculate_sweep_frequency(&mut self) -> u16 {
        let delta = self.shadow_frequency >> self.sweep_shift;
        
        let new_freq = if self.sweep_direction {
            self.shadow_frequency.wrapping_sub(delta)
        } else {
            self.shadow_frequency.wrapping_add(delta)
        };
        
        if new_freq > 2047 {
            self.enabled = false;
        }
        
        new_freq
    }
    
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        
        self.frequency_timer = (2048 - self.frequency as u32) * 4;
        self.envelope_timer = self.envelope_period;
        self.volume = self.initial_volume;
        
        // Sweep
        self.shadow_frequency = self.frequency;
        self.sweep_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };
        self.sweep_enabled = self.sweep_period > 0 || self.sweep_shift > 0;
        
        if self.sweep_shift > 0 {
            self.calculate_sweep_frequency();
        }
    }
}

/// Square wave channel (Channel 2)
struct Channel2 {
    enabled: bool,
    dac_enabled: bool,
    length_counter: u8,
    length_enabled: bool,
    frequency: u16,
    frequency_timer: u32,
    duty_position: u8,
    duty: u8,
    volume: u8,
    initial_volume: u8,
    envelope_timer: u8,
    envelope_direction: bool,
    envelope_period: u8,
}

impl Default for Channel2 {
    fn default() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            length_counter: 0,
            length_enabled: false,
            frequency: 0,
            frequency_timer: 0,
            duty_position: 0,
            duty: 0,
            volume: 0,
            initial_volume: 0,
            envelope_timer: 0,
            envelope_direction: false,
            envelope_period: 0,
        }
    }
}

impl Channel2 {
    fn step(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }
        
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency as u32) * 4;
            self.duty_position = (self.duty_position + 1) & 7;
        }
    }
    
    fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }
        
        let duty_table: [[u8; 8]; 4] = [
            [0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 1, 1, 1],
            [0, 1, 1, 1, 1, 1, 1, 0],
        ];
        
        let sample = duty_table[self.duty as usize][self.duty_position as usize];
        let volume = self.volume as f32 / 15.0;
        
        if sample == 1 { volume } else { -volume }
    }
    
    fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
    
    fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
            return;
        }
        
        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }
        
        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_period;
            
            if self.envelope_direction && self.volume < 15 {
                self.volume += 1;
            } else if !self.envelope_direction && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
    
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        
        self.frequency_timer = (2048 - self.frequency as u32) * 4;
        self.envelope_timer = self.envelope_period;
        self.volume = self.initial_volume;
    }
}

/// Wave channel (Channel 3)
struct Channel3 {
    enabled: bool,
    dac_enabled: bool,
    length_counter: u16,
    length_enabled: bool,
    frequency: u16,
    frequency_timer: u32,
    volume_code: u8,
    sample_index: u8,
    wave_ram: [u8; 16],
}

impl Default for Channel3 {
    fn default() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            length_counter: 0,
            length_enabled: false,
            frequency: 0,
            frequency_timer: 0,
            volume_code: 0,
            sample_index: 0,
            wave_ram: [0; 16],
        }
    }
}

impl Channel3 {
    fn step(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }
        
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency as u32) * 2;
            self.sample_index = (self.sample_index + 1) & 31;
        }
    }
    
    fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }
        
        let byte = self.wave_ram[(self.sample_index / 2) as usize];
        let sample = if self.sample_index & 1 == 0 {
            byte >> 4
        } else {
            byte & 0x0F
        };
        
        let shifted = match self.volume_code {
            0 => 0,
            1 => sample,
            2 => sample >> 1,
            3 => sample >> 2,
            _ => 0,
        };
        
        (shifted as f32 / 7.5) - 1.0
    }
    
    fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
    
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        
        if self.length_counter == 0 {
            self.length_counter = 256;
        }
        
        self.frequency_timer = (2048 - self.frequency as u32) * 2;
        self.sample_index = 0;
    }
}

/// Noise channel (Channel 4)
struct Channel4 {
    enabled: bool,
    dac_enabled: bool,
    length_counter: u8,
    length_enabled: bool,
    frequency_timer: u32,
    volume: u8,
    initial_volume: u8,
    envelope_timer: u8,
    envelope_direction: bool,
    envelope_period: u8,
    lfsr: u16,
    clock_shift: u8,
    width_mode: bool,
    divisor_code: u8,
}

impl Default for Channel4 {
    fn default() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            length_counter: 0,
            length_enabled: false,
            frequency_timer: 0,
            volume: 0,
            initial_volume: 0,
            envelope_timer: 0,
            envelope_direction: false,
            envelope_period: 0,
            lfsr: 0x7FFF,
            clock_shift: 0,
            width_mode: false,
            divisor_code: 0,
        }
    }
}

impl Channel4 {
    fn step(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }
        
        if self.frequency_timer == 0 {
            let divisor = match self.divisor_code {
                0 => 8,
                n => (n as u32) * 16,
            };
            self.frequency_timer = divisor << self.clock_shift;
            
            // Clock LFSR
            let xor_result = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);
            
            if self.width_mode {
                self.lfsr = (self.lfsr & !0x40) | (xor_result << 6);
            }
        }
    }
    
    fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }
        
        let sample = !(self.lfsr & 1) as u8;
        let volume = self.volume as f32 / 15.0;
        
        if sample == 1 { volume } else { -volume }
    }
    
    fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
    
    fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
            return;
        }
        
        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }
        
        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_period;
            
            if self.envelope_direction && self.volume < 15 {
                self.volume += 1;
            } else if !self.envelope_direction && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
    
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        
        let divisor = match self.divisor_code {
            0 => 8,
            n => (n as u32) * 16,
        };
        self.frequency_timer = divisor << self.clock_shift;
        self.envelope_timer = self.envelope_period;
        self.volume = self.initial_volume;
        self.lfsr = 0x7FFF;
    }
}

/// Audio Processing Unit
pub struct Apu {
    enabled: bool,
    
    channel1: Channel1,
    channel2: Channel2,
    channel3: Channel3,
    channel4: Channel4,
    
    // Output control
    left_volume: u8,
    right_volume: u8,
    left_enables: u8,
    right_enables: u8,
    
    // Frame sequencer
    frame_sequencer_timer: u32,
    frame_sequencer_step: u8,
    
    // Sample generation
    sample_timer: u32,
    output_buffer: Vec<f32>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: true,
            channel1: Channel1::default(),
            channel2: Channel2::default(),
            channel3: Channel3::default(),
            channel4: Channel4::default(),
            left_volume: 7,
            right_volume: 7,
            left_enables: 0xFF,
            right_enables: 0xFF,
            frame_sequencer_timer: 0,
            frame_sequencer_step: 0,
            sample_timer: 0,
            output_buffer: Vec::with_capacity(4096),
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    pub fn step(&mut self, cycles: u32) {
        if !self.enabled {
            return;
        }
        
        for _ in 0..cycles {
            // Step channels
            self.channel1.step();
            self.channel2.step();
            self.channel3.step();
            self.channel4.step();
            
            // Frame sequencer
            self.frame_sequencer_timer += 1;
            if self.frame_sequencer_timer >= FRAME_SEQUENCER_RATE {
                self.frame_sequencer_timer = 0;
                self.clock_frame_sequencer();
            }
            
            // Generate samples
            self.sample_timer += 1;
            if self.sample_timer >= CYCLES_PER_SAMPLE {
                self.sample_timer = 0;
                self.generate_sample();
            }
        }
    }
    
    fn clock_frame_sequencer(&mut self) {
        match self.frame_sequencer_step {
            0 => {
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
            }
            2 => {
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
                self.channel1.clock_sweep();
            }
            4 => {
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
            }
            6 => {
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
                self.channel1.clock_sweep();
            }
            7 => {
                self.channel1.clock_envelope();
                self.channel2.clock_envelope();
                self.channel4.clock_envelope();
            }
            _ => {}
        }
        
        self.frame_sequencer_step = (self.frame_sequencer_step + 1) & 7;
    }
    
    fn generate_sample(&mut self) {
        let ch1 = self.channel1.output();
        let ch2 = self.channel2.output();
        let ch3 = self.channel3.output();
        let ch4 = self.channel4.output();
        
        // Mix channels
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        
        if self.left_enables & 0x01 != 0 { left += ch1; }
        if self.left_enables & 0x02 != 0 { left += ch2; }
        if self.left_enables & 0x04 != 0 { left += ch3; }
        if self.left_enables & 0x08 != 0 { left += ch4; }
        
        if self.right_enables & 0x10 != 0 { right += ch1; }
        if self.right_enables & 0x20 != 0 { right += ch2; }
        if self.right_enables & 0x40 != 0 { right += ch3; }
        if self.right_enables & 0x80 != 0 { right += ch4; }
        
        // Apply master volume
        left *= (self.left_volume as f32 + 1.0) / 32.0;
        right *= (self.right_volume as f32 + 1.0) / 32.0;
        
        // Clamp
        left = left.clamp(-1.0, 1.0);
        right = right.clamp(-1.0, 1.0);
        
        self.output_buffer.push(left);
        self.output_buffer.push(right);
    }
    
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            // NR10 - Channel 1 Sweep
            0xFF10 => {
                0x80 | (self.channel1.sweep_period << 4)
                    | (if self.channel1.sweep_direction { 0x08 } else { 0 })
                    | self.channel1.sweep_shift
            }
            // NR11 - Channel 1 Length/Duty
            0xFF11 => (self.channel1.duty << 6) | 0x3F,
            // NR12 - Channel 1 Envelope
            0xFF12 => {
                (self.channel1.initial_volume << 4)
                    | (if self.channel1.envelope_direction { 0x08 } else { 0 })
                    | self.channel1.envelope_period
            }
            // NR13 - Channel 1 Frequency low (write only)
            0xFF13 => 0xFF,
            // NR14 - Channel 1 Frequency high
            0xFF14 => 0xBF | (if self.channel1.length_enabled { 0x40 } else { 0 }),
            
            // NR21 - Channel 2 Length/Duty
            0xFF16 => (self.channel2.duty << 6) | 0x3F,
            // NR22 - Channel 2 Envelope
            0xFF17 => {
                (self.channel2.initial_volume << 4)
                    | (if self.channel2.envelope_direction { 0x08 } else { 0 })
                    | self.channel2.envelope_period
            }
            // NR23 - Channel 2 Frequency low (write only)
            0xFF18 => 0xFF,
            // NR24 - Channel 2 Frequency high
            0xFF19 => 0xBF | (if self.channel2.length_enabled { 0x40 } else { 0 }),
            
            // NR30 - Channel 3 DAC
            0xFF1A => 0x7F | (if self.channel3.dac_enabled { 0x80 } else { 0 }),
            // NR31 - Channel 3 Length (write only)
            0xFF1B => 0xFF,
            // NR32 - Channel 3 Volume
            0xFF1C => 0x9F | (self.channel3.volume_code << 5),
            // NR33 - Channel 3 Frequency low (write only)
            0xFF1D => 0xFF,
            // NR34 - Channel 3 Frequency high
            0xFF1E => 0xBF | (if self.channel3.length_enabled { 0x40 } else { 0 }),
            
            // NR41 - Channel 4 Length (write only)
            0xFF20 => 0xFF,
            // NR42 - Channel 4 Envelope
            0xFF21 => {
                (self.channel4.initial_volume << 4)
                    | (if self.channel4.envelope_direction { 0x08 } else { 0 })
                    | self.channel4.envelope_period
            }
            // NR43 - Channel 4 Polynomial counter
            0xFF22 => {
                (self.channel4.clock_shift << 4)
                    | (if self.channel4.width_mode { 0x08 } else { 0 })
                    | self.channel4.divisor_code
            }
            // NR44 - Channel 4 Control
            0xFF23 => 0xBF | (if self.channel4.length_enabled { 0x40 } else { 0 }),
            
            // NR50 - Master volume
            0xFF24 => (self.left_volume << 4) | self.right_volume,
            
            // NR51 - Sound panning
            0xFF25 => self.left_enables | self.right_enables,
            
            // NR52 - Sound on/off
            0xFF26 => {
                0x70
                    | (if self.enabled { 0x80 } else { 0 })
                    | (if self.channel4.enabled { 0x08 } else { 0 })
                    | (if self.channel3.enabled { 0x04 } else { 0 })
                    | (if self.channel2.enabled { 0x02 } else { 0 })
                    | (if self.channel1.enabled { 0x01 } else { 0 })
            }
            
            // Wave RAM
            0xFF30..=0xFF3F => self.channel3.wave_ram[(addr - 0xFF30) as usize],
            
            _ => 0xFF,
        }
    }
    
    pub fn write_register(&mut self, addr: u16, value: u8) {
        if !self.enabled && addr != 0xFF26 && !(0xFF30..=0xFF3F).contains(&addr) {
            return;
        }
        
        match addr {
            // NR10 - Channel 1 Sweep
            0xFF10 => {
                self.channel1.sweep_period = (value >> 4) & 0x07;
                self.channel1.sweep_direction = value & 0x08 != 0;
                self.channel1.sweep_shift = value & 0x07;
            }
            // NR11 - Channel 1 Length/Duty
            0xFF11 => {
                self.channel1.duty = (value >> 6) & 0x03;
                self.channel1.length_counter = 64 - (value & 0x3F);
            }
            // NR12 - Channel 1 Envelope
            0xFF12 => {
                self.channel1.initial_volume = (value >> 4) & 0x0F;
                self.channel1.envelope_direction = value & 0x08 != 0;
                self.channel1.envelope_period = value & 0x07;
                self.channel1.dac_enabled = value & 0xF8 != 0;
                if !self.channel1.dac_enabled {
                    self.channel1.enabled = false;
                }
            }
            // NR13 - Channel 1 Frequency low
            0xFF13 => {
                self.channel1.frequency = (self.channel1.frequency & 0x700) | value as u16;
            }
            // NR14 - Channel 1 Frequency high
            0xFF14 => {
                self.channel1.frequency = (self.channel1.frequency & 0xFF) | ((value as u16 & 0x07) << 8);
                self.channel1.length_enabled = value & 0x40 != 0;
                if value & 0x80 != 0 {
                    self.channel1.trigger();
                }
            }
            
            // NR21 - Channel 2 Length/Duty
            0xFF16 => {
                self.channel2.duty = (value >> 6) & 0x03;
                self.channel2.length_counter = 64 - (value & 0x3F);
            }
            // NR22 - Channel 2 Envelope
            0xFF17 => {
                self.channel2.initial_volume = (value >> 4) & 0x0F;
                self.channel2.envelope_direction = value & 0x08 != 0;
                self.channel2.envelope_period = value & 0x07;
                self.channel2.dac_enabled = value & 0xF8 != 0;
                if !self.channel2.dac_enabled {
                    self.channel2.enabled = false;
                }
            }
            // NR23 - Channel 2 Frequency low
            0xFF18 => {
                self.channel2.frequency = (self.channel2.frequency & 0x700) | value as u16;
            }
            // NR24 - Channel 2 Frequency high
            0xFF19 => {
                self.channel2.frequency = (self.channel2.frequency & 0xFF) | ((value as u16 & 0x07) << 8);
                self.channel2.length_enabled = value & 0x40 != 0;
                if value & 0x80 != 0 {
                    self.channel2.trigger();
                }
            }
            
            // NR30 - Channel 3 DAC
            0xFF1A => {
                self.channel3.dac_enabled = value & 0x80 != 0;
                if !self.channel3.dac_enabled {
                    self.channel3.enabled = false;
                }
            }
            // NR31 - Channel 3 Length
            0xFF1B => {
                self.channel3.length_counter = 256 - value as u16;
            }
            // NR32 - Channel 3 Volume
            0xFF1C => {
                self.channel3.volume_code = (value >> 5) & 0x03;
            }
            // NR33 - Channel 3 Frequency low
            0xFF1D => {
                self.channel3.frequency = (self.channel3.frequency & 0x700) | value as u16;
            }
            // NR34 - Channel 3 Frequency high
            0xFF1E => {
                self.channel3.frequency = (self.channel3.frequency & 0xFF) | ((value as u16 & 0x07) << 8);
                self.channel3.length_enabled = value & 0x40 != 0;
                if value & 0x80 != 0 {
                    self.channel3.trigger();
                }
            }
            
            // NR41 - Channel 4 Length
            0xFF20 => {
                self.channel4.length_counter = 64 - (value & 0x3F);
            }
            // NR42 - Channel 4 Envelope
            0xFF21 => {
                self.channel4.initial_volume = (value >> 4) & 0x0F;
                self.channel4.envelope_direction = value & 0x08 != 0;
                self.channel4.envelope_period = value & 0x07;
                self.channel4.dac_enabled = value & 0xF8 != 0;
                if !self.channel4.dac_enabled {
                    self.channel4.enabled = false;
                }
            }
            // NR43 - Channel 4 Polynomial counter
            0xFF22 => {
                self.channel4.clock_shift = (value >> 4) & 0x0F;
                self.channel4.width_mode = value & 0x08 != 0;
                self.channel4.divisor_code = value & 0x07;
            }
            // NR44 - Channel 4 Control
            0xFF23 => {
                self.channel4.length_enabled = value & 0x40 != 0;
                if value & 0x80 != 0 {
                    self.channel4.trigger();
                }
            }
            
            // NR50 - Master volume
            0xFF24 => {
                self.left_volume = (value >> 4) & 0x07;
                self.right_volume = value & 0x07;
            }
            
            // NR51 - Sound panning
            0xFF25 => {
                self.left_enables = value & 0x0F;
                self.right_enables = value & 0xF0;
            }
            
            // NR52 - Sound on/off
            0xFF26 => {
                let was_enabled = self.enabled;
                self.enabled = value & 0x80 != 0;
                
                if was_enabled && !self.enabled {
                    // Clear all registers when disabled
                    self.channel1 = Channel1::default();
                    self.channel2 = Channel2::default();
                    self.channel3 = Channel3::default();
                    self.channel4 = Channel4::default();
                }
            }
            
            // Wave RAM
            0xFF30..=0xFF3F => {
                self.channel3.wave_ram[(addr - 0xFF30) as usize] = value;
            }
            
            _ => {}
        }
    }
    
    pub fn output_buffer(&self) -> &[f32] {
        &self.output_buffer
    }
    
    pub fn clear_buffer(&mut self) {
        self.output_buffer.clear();
    }
    
    pub fn state(&self) -> ApuState {
        ApuState {
            enabled: self.enabled,
            frame_sequencer_step: self.frame_sequencer_step,
            channel1: Channel1State {
                enabled: self.channel1.enabled,
                dac_enabled: self.channel1.dac_enabled,
                length_counter: self.channel1.length_counter,
                frequency: self.channel1.frequency,
                duty: self.channel1.duty,
                volume: self.channel1.volume,
                envelope_timer: self.channel1.envelope_timer,
                envelope_direction: self.channel1.envelope_direction,
                envelope_period: self.channel1.envelope_period,
                sweep_timer: self.channel1.sweep_timer,
                sweep_period: self.channel1.sweep_period,
                sweep_direction: self.channel1.sweep_direction,
                sweep_shift: self.channel1.sweep_shift,
                sweep_enabled: self.channel1.sweep_enabled,
                shadow_frequency: self.channel1.shadow_frequency,
            },
            channel2: Channel2State {
                enabled: self.channel2.enabled,
                dac_enabled: self.channel2.dac_enabled,
                length_counter: self.channel2.length_counter,
                frequency: self.channel2.frequency,
                duty: self.channel2.duty,
                volume: self.channel2.volume,
                envelope_timer: self.channel2.envelope_timer,
                envelope_direction: self.channel2.envelope_direction,
                envelope_period: self.channel2.envelope_period,
            },
            channel3: Channel3State {
                enabled: self.channel3.enabled,
                dac_enabled: self.channel3.dac_enabled,
                length_counter: self.channel3.length_counter,
                frequency: self.channel3.frequency,
                volume_code: self.channel3.volume_code,
                sample_index: self.channel3.sample_index,
            },
            channel4: Channel4State {
                enabled: self.channel4.enabled,
                dac_enabled: self.channel4.dac_enabled,
                length_counter: self.channel4.length_counter,
                volume: self.channel4.volume,
                envelope_timer: self.channel4.envelope_timer,
                envelope_direction: self.channel4.envelope_direction,
                envelope_period: self.channel4.envelope_period,
                lfsr: self.channel4.lfsr,
                clock_shift: self.channel4.clock_shift,
                width_mode: self.channel4.width_mode,
                divisor_code: self.channel4.divisor_code,
            },
        }
    }
    
    pub fn load_state(&mut self, state: ApuState) {
        self.enabled = state.enabled;
        self.frame_sequencer_step = state.frame_sequencer_step;
        
        // Channel 1
        self.channel1.enabled = state.channel1.enabled;
        self.channel1.dac_enabled = state.channel1.dac_enabled;
        self.channel1.length_counter = state.channel1.length_counter;
        self.channel1.frequency = state.channel1.frequency;
        self.channel1.duty = state.channel1.duty;
        self.channel1.volume = state.channel1.volume;
        self.channel1.envelope_timer = state.channel1.envelope_timer;
        self.channel1.envelope_direction = state.channel1.envelope_direction;
        self.channel1.envelope_period = state.channel1.envelope_period;
        self.channel1.sweep_timer = state.channel1.sweep_timer;
        self.channel1.sweep_period = state.channel1.sweep_period;
        self.channel1.sweep_direction = state.channel1.sweep_direction;
        self.channel1.sweep_shift = state.channel1.sweep_shift;
        self.channel1.sweep_enabled = state.channel1.sweep_enabled;
        self.channel1.shadow_frequency = state.channel1.shadow_frequency;
        
        // Channel 2
        self.channel2.enabled = state.channel2.enabled;
        self.channel2.dac_enabled = state.channel2.dac_enabled;
        self.channel2.length_counter = state.channel2.length_counter;
        self.channel2.frequency = state.channel2.frequency;
        self.channel2.duty = state.channel2.duty;
        self.channel2.volume = state.channel2.volume;
        self.channel2.envelope_timer = state.channel2.envelope_timer;
        self.channel2.envelope_direction = state.channel2.envelope_direction;
        self.channel2.envelope_period = state.channel2.envelope_period;
        
        // Channel 3
        self.channel3.enabled = state.channel3.enabled;
        self.channel3.dac_enabled = state.channel3.dac_enabled;
        self.channel3.length_counter = state.channel3.length_counter;
        self.channel3.frequency = state.channel3.frequency;
        self.channel3.volume_code = state.channel3.volume_code;
        self.channel3.sample_index = state.channel3.sample_index;
        
        // Channel 4
        self.channel4.enabled = state.channel4.enabled;
        self.channel4.dac_enabled = state.channel4.dac_enabled;
        self.channel4.length_counter = state.channel4.length_counter;
        self.channel4.volume = state.channel4.volume;
        self.channel4.envelope_timer = state.channel4.envelope_timer;
        self.channel4.envelope_direction = state.channel4.envelope_direction;
        self.channel4.envelope_period = state.channel4.envelope_period;
        self.channel4.lfsr = state.channel4.lfsr;
        self.channel4.clock_shift = state.channel4.clock_shift;
        self.channel4.width_mode = state.channel4.width_mode;
        self.channel4.divisor_code = state.channel4.divisor_code;
    }
}
