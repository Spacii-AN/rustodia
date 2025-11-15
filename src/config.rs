use device_query::Keycode;

// Shared configuration that can be modified by GUI
#[derive(Clone)]
pub struct SharedConfig {
    // Timing settings
    pub fps: f64,
    pub jump_delay_ms: f64,
    pub aim_melee_delay_ms: u64,
    pub melee_hold_time_ms: u64,
    pub use_emote_formula: bool,
    pub emote_preparation_delay_manual_ms: u64,
    pub rapid_fire_duration_ms: u64,
    pub rapid_fire_click_delay_ms: u64,
    pub sequence_end_delay_ms: u64,
    pub loop_delay_ms: u64,
    pub rapid_click_count: usize,
    pub rapid_click_delay_ms: u64,
    
    // Keybind settings (as strings for GUI, converted to Keycode when needed)
    pub melee_key: String,
    pub jump_key: String,
    pub emote_key: String,
    pub rapid_click_key: String,
    pub aim_button: usize,
    pub fire_button: usize,
    pub macro_button: usize,
    pub enable_macro_alt: bool,
    pub macro_alt_button: usize,
}

impl Default for SharedConfig {
    fn default() -> Self {
        Self {
            fps: 160.0,
            jump_delay_ms: 1100.0,
            aim_melee_delay_ms: 25,
            melee_hold_time_ms: 50,
            use_emote_formula: true,
            emote_preparation_delay_manual_ms: 100,
            rapid_fire_duration_ms: 230,
            rapid_fire_click_delay_ms: 1,
            sequence_end_delay_ms: 50,
            loop_delay_ms: 1,
            rapid_click_count: 10,
            rapid_click_delay_ms: 50,
            melee_key: "E".to_string(),
            jump_key: "Space".to_string(),
            emote_key: ".".to_string(),
            rapid_click_key: "J".to_string(),
            aim_button: 2,
            fire_button: 1,
            macro_button: 8,  // Side button 1 (button8 in pynput)
            enable_macro_alt: true,
            macro_alt_button: 9,  // Side button 2 (button9 in pynput)
        }
    }
}

impl SharedConfig {
    // Convert Keycode to string representation
    pub fn keycode_to_string(keycode: Keycode) -> String {
        // Use Debug formatting and clean it up
        let debug_str = format!("{:?}", keycode);
        
        // Handle common cases
        match keycode {
            Keycode::Space => "Space".to_string(),
            Keycode::A => "A".to_string(),
            Keycode::B => "B".to_string(),
            Keycode::C => "C".to_string(),
            Keycode::D => "D".to_string(),
            Keycode::E => "E".to_string(),
            Keycode::F => "F".to_string(),
            Keycode::G => "G".to_string(),
            Keycode::H => "H".to_string(),
            Keycode::I => "I".to_string(),
            Keycode::J => "J".to_string(),
            Keycode::K => "K".to_string(),
            Keycode::L => "L".to_string(),
            Keycode::M => "M".to_string(),
            Keycode::N => "N".to_string(),
            Keycode::O => "O".to_string(),
            Keycode::P => "P".to_string(),
            Keycode::Q => "Q".to_string(),
            Keycode::R => "R".to_string(),
            Keycode::S => "S".to_string(),
            Keycode::T => "T".to_string(),
            Keycode::U => "U".to_string(),
            Keycode::V => "V".to_string(),
            Keycode::W => "W".to_string(),
            Keycode::X => "X".to_string(),
            Keycode::Y => "Y".to_string(),
            Keycode::Z => "Z".to_string(),
            Keycode::Key0 => "0".to_string(),
            Keycode::Key1 => "1".to_string(),
            Keycode::Key2 => "2".to_string(),
            Keycode::Key3 => "3".to_string(),
            Keycode::Key4 => "4".to_string(),
            Keycode::Key5 => "5".to_string(),
            Keycode::Key6 => "6".to_string(),
            Keycode::Key7 => "7".to_string(),
            Keycode::Key8 => "8".to_string(),
            Keycode::Key9 => "9".to_string(),
            Keycode::F1 => "F1".to_string(),
            Keycode::F2 => "F2".to_string(),
            Keycode::F3 => "F3".to_string(),
            Keycode::F4 => "F4".to_string(),
            Keycode::F5 => "F5".to_string(),
            Keycode::F6 => "F6".to_string(),
            Keycode::F7 => "F7".to_string(),
            Keycode::F8 => "F8".to_string(),
            Keycode::F9 => "F9".to_string(),
            Keycode::F10 => "F10".to_string(),
            Keycode::F11 => "F11".to_string(),
            Keycode::F12 => "F12".to_string(),
            Keycode::Dot => ".".to_string(),
            _ => {
                // Clean up Debug output: remove "Keycode::" prefix if present
                debug_str.replace("Keycode::", "")
            }
        }
    }
    
    // Convert string key to Keycode
    pub fn keycode_from_string(s: &str) -> Keycode {
        // Try to match the string (case-insensitive)
        let upper = s.to_uppercase();
        match upper.as_str() {
            "SPACE" => Keycode::Space,
            "A" => Keycode::A,
            "B" => Keycode::B,
            "C" => Keycode::C,
            "D" => Keycode::D,
            "E" => Keycode::E,
            "F" => Keycode::F,
            "G" => Keycode::G,
            "H" => Keycode::H,
            "I" => Keycode::I,
            "J" => Keycode::J,
            "K" => Keycode::K,
            "L" => Keycode::L,
            "M" => Keycode::M,
            "N" => Keycode::N,
            "O" => Keycode::O,
            "P" => Keycode::P,
            "Q" => Keycode::Q,
            "R" => Keycode::R,
            "S" => Keycode::S,
            "T" => Keycode::T,
            "U" => Keycode::U,
            "V" => Keycode::V,
            "W" => Keycode::W,
            "X" => Keycode::X,
            "Y" => Keycode::Y,
            "Z" => Keycode::Z,
            "0" => Keycode::Key0,
            "1" => Keycode::Key1,
            "2" => Keycode::Key2,
            "3" => Keycode::Key3,
            "4" => Keycode::Key4,
            "5" => Keycode::Key5,
            "6" => Keycode::Key6,
            "7" => Keycode::Key7,
            "8" => Keycode::Key8,
            "9" => Keycode::Key9,
            "F1" => Keycode::F1,
            "F2" => Keycode::F2,
            "F3" => Keycode::F3,
            "F4" => Keycode::F4,
            "F5" => Keycode::F5,
            "F6" => Keycode::F6,
            "F7" => Keycode::F7,
            "F8" => Keycode::F8,
            "F9" => Keycode::F9,
            "F10" => Keycode::F10,
            "F11" => Keycode::F11,
            "F12" => Keycode::F12,
            "." | "PERIOD" | "DOT" => Keycode::Dot,
            _ => {
                // Try to parse as Debug format (e.g., "Keycode::E" or just "E")
                if upper.starts_with("KEYCODE::") {
                    let key_name = &upper[9..];
                    Self::keycode_from_string(key_name)
                } else {
                    Keycode::E // Default fallback
                }
            }
        }
    }
    
    // Convert to internal Keybinds structure
    pub fn to_keybinds(&self) -> crate::Keybinds {
        crate::Keybinds {
            melee: Self::keycode_from_string(&self.melee_key),
            jump: Self::keycode_from_string(&self.jump_key),
            aim: self.aim_button,
            fire: self.fire_button,
            emote: Self::keycode_from_string(&self.emote_key),
            macro_button: self.macro_button,
            macro_alt: if self.enable_macro_alt { Some(self.macro_alt_button) } else { None },
            rapid_click: Self::keycode_from_string(&self.rapid_click_key),
        }
    }
    
    // Get timing durations
    pub fn double_jump_delay(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(self.jump_delay_ms / self.fps / 1000.0)
    }
    
    pub fn emote_preparation_delay(&self) -> std::time::Duration {
        if self.use_emote_formula {
            let raw_delay_ms = (-26.0 * self.fps.ln() + 245.0).max(0.0) as u64;
            std::time::Duration::from_millis(raw_delay_ms)
        } else {
            std::time::Duration::from_millis(self.emote_preparation_delay_manual_ms)
        }
    }
}

