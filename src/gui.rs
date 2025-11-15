use eframe::egui;
use crate::config::SharedConfig;
use device_query::{DeviceQuery, DeviceState};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
use evdev::Device;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CaptureTarget {
    None,
    MeleeKey,
    JumpKey,
    EmoteKey,
    RapidClickKey,
    AimButton,
    FireButton,
    MacroButton,
    MacroAltButton,
}

pub struct MacroApp {
    config: std::sync::Arc<std::sync::Mutex<SharedConfig>>,
    capture_target: Arc<Mutex<CaptureTarget>>,
}


impl MacroApp {
    pub fn new(config: std::sync::Arc<std::sync::Mutex<SharedConfig>>) -> Self {
        let capture_target = Arc::new(Mutex::new(CaptureTarget::None));
        
        #[cfg(target_os = "linux")]
        {
            eprintln!("⚠️  Note: Side buttons (button8/button9) may not be detected by device_query on Linux");
            eprintln!("   We'll try to use evdev as a fallback for side button detection");
        }
        
        // Start background thread for key capture
        let config_clone = Arc::clone(&config);
        let capture_target_clone = Arc::clone(&capture_target);
        
        thread::spawn(move || {
            eprintln!("GUI key capture thread started");
            let device_state = DeviceState::new();
            let mut last_keys = std::collections::HashSet::new();
            let mut last_mouse_buttons = Vec::new();
            let mut capture_started = false;
            let mut loop_count = 0;
            
            #[cfg(target_os = "linux")]
            let mut evdev_listener: Option<Device> = {
                // Try to open a mouse device for evdev monitoring
                use std::fs;
                use std::path::Path;
                let input_dir = Path::new("/dev/input");
                let mut found_device = None;
                if let Ok(entries) = fs::read_dir(input_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with("event") {
                                match Device::open(&path) {
                                    Ok(device) => {
                                        // Check if device has mouse buttons (including side buttons)
                                        let name_lower = device.name().unwrap_or_default().to_lowercase();
                                        if name_lower.contains("mouse") || name_lower.contains("pointer") {
                                            eprintln!("Found potential mouse device: {} ({})", name, device.name().unwrap_or_default());
                                            found_device = Some(device);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        // Permission denied is expected if not in input group
                                        // We'll just skip this device and continue
                                        if !e.to_string().contains("Permission denied") {
                                            eprintln!("Warning: Could not open {}: {}", name, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if found_device.is_none() {
                    eprintln!("⚠️  Could not open evdev device for side button detection.");
                    eprintln!("   This is normal if you're not in the 'input' group.");
                    eprintln!("   Side buttons may still work via device_query, or add yourself to input group:");
                    eprintln!("   sudo usermod -aG input $USER  (then log out/in)");
                }
                found_device
            };
            
            loop {
                thread::sleep(Duration::from_millis(10)); // Poll every 10ms
                loop_count += 1;
                
                // Log every 5 seconds that thread is alive
                if loop_count % 500 == 0 {
                    eprintln!("GUI capture thread alive (loop {})", loop_count);
                }
                
                let current_target = *capture_target_clone.lock().unwrap();
                if current_target == CaptureTarget::None {
                    last_keys.clear();
                    last_mouse_buttons.clear();
                    capture_started = false;
                    continue;
                }
                
                // Small delay after starting capture to avoid capturing the click that started it
                if !capture_started {
                    capture_started = true;
                    eprintln!("Starting key capture for: {:?}", current_target);
                    let keys = device_state.get_keys();
                    last_keys = keys.iter().cloned().collect();
                    let mouse = device_state.get_mouse();
                    last_mouse_buttons = mouse.button_pressed.clone();
                    eprintln!("Initial state - Keys pressed: {:?}, Mouse buttons: {:?}", 
                        keys.len(), 
                        mouse.button_pressed.iter().enumerate().filter(|(_, &p)| p).map(|(i, _)| i).collect::<Vec<_>>()
                    );
                    thread::sleep(Duration::from_millis(200)); // Wait 200ms before starting to capture
                    continue;
                }
                
                let keys = device_state.get_keys();
                let mouse = device_state.get_mouse();
                
                // On Linux, also check evdev for side buttons
                #[cfg(target_os = "linux")]
                {
                    if let Some(ref mut evdev_device) = evdev_listener {
                        if matches!(current_target, CaptureTarget::AimButton | CaptureTarget::FireButton | CaptureTarget::MacroButton | CaptureTarget::MacroAltButton) {
                            // Try to read events from evdev (non-blocking)
                            match evdev_device.fetch_events() {
                                Ok(events) => {
                                    for event in events {
                                        // Check if this is a key event
                                        if event.event_type().0 == 1 { // EV_KEY = 1
                                            let code = event.code();
                                            // BTN_SIDE = 0x113 (275), BTN_EXTRA = 0x114 (276)
                                            // But we need to check the actual key code
                                            if code == 0x113 || code == 275 {
                                                eprintln!("✅ Side button 1 (BTN_SIDE, code {}) detected via evdev!", code);
                                                let mut config = config_clone.lock().unwrap();
                                                match current_target {
                                                    CaptureTarget::AimButton => config.aim_button = 8,
                                                    CaptureTarget::FireButton => config.fire_button = 8,
                                                    CaptureTarget::MacroButton => config.macro_button = 8,
                                                    CaptureTarget::MacroAltButton => config.macro_alt_button = 8,
                                                    _ => {}
                                                }
                                                *capture_target_clone.lock().unwrap() = CaptureTarget::None;
                                                capture_started = false;
                                                continue;
                                            } else if code == 0x114 || code == 276 {
                                                eprintln!("✅ Side button 2 (BTN_EXTRA, code {}) detected via evdev!", code);
                                                let mut config = config_clone.lock().unwrap();
                                                match current_target {
                                                    CaptureTarget::AimButton => config.aim_button = 9,
                                                    CaptureTarget::FireButton => config.fire_button = 9,
                                                    CaptureTarget::MacroButton => config.macro_button = 9,
                                                    CaptureTarget::MacroAltButton => config.macro_alt_button = 9,
                                                    _ => {}
                                                }
                                                *capture_target_clone.lock().unwrap() = CaptureTarget::None;
                                                capture_started = false;
                                                continue;
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // Device might have been disconnected or no events available
                                    // This is normal, just continue
                                }
                            }
                        }
                    }
                }
                
                // Log mouse button state when capturing mouse buttons (but only when state changes to avoid spam)
                if matches!(current_target, CaptureTarget::AimButton | CaptureTarget::FireButton | CaptureTarget::MacroButton | CaptureTarget::MacroAltButton) {
                    let current_pressed: Vec<usize> = mouse.button_pressed.iter().enumerate()
                        .filter(|(_, &p)| p)
                        .map(|(i, _)| i)
                        .collect();
                    let last_pressed: Vec<usize> = last_mouse_buttons.iter().enumerate()
                        .filter(|(_, p)| **p)
                        .map(|(i, _)| i)
                        .collect();
                    
                    // Only log when button state changes
                    if current_pressed != last_pressed {
                        eprintln!("🔍 Mouse button state changed - Pressed indices: {:?}, Array length: {}, Full array: {:?}", 
                            current_pressed, 
                            mouse.button_pressed.len(),
                            mouse.button_pressed
                        );
                        eprintln!("   NOTE: If side buttons aren't showing, device_query may not support them on Linux");
                        eprintln!("   Side buttons might need to be detected via evdev or xdotool instead");
                    }
                }
                
                // Debug: log when keys change
                if keys.len() != last_keys.len() || !keys.iter().all(|k| last_keys.contains(k)) {
                    let new_keys: Vec<_> = keys.iter().filter(|k| !last_keys.contains(k)).collect();
                    if !new_keys.is_empty() {
                        eprintln!("New keys detected: {:?}", new_keys);
                    }
                }
                
                // Debug: log when mouse buttons change
                let current_pressed: Vec<usize> = mouse.button_pressed.iter().enumerate()
                    .filter(|(_, &p)| p)
                    .map(|(i, _)| i)
                    .collect();
                let last_pressed: Vec<usize> = last_mouse_buttons.iter().enumerate()
                    .filter(|(_, p)| **p)
                    .map(|(i, _)| i)
                    .collect();
                if current_pressed != last_pressed {
                    let new_buttons: Vec<usize> = current_pressed.iter()
                        .filter(|&&i| !last_pressed.contains(&i))
                        .copied()
                        .collect();
                    if !new_buttons.is_empty() {
                        eprintln!("New mouse buttons detected: {:?} (full array: {:?})", new_buttons, mouse.button_pressed);
                    }
                }
                
                // Check for Escape to cancel capture
                use device_query::Keycode;
                if keys.contains(&Keycode::Escape) && !last_keys.contains(&Keycode::Escape) {
                    eprintln!("Escape pressed, canceling capture");
                    *capture_target_clone.lock().unwrap() = CaptureTarget::None;
                    capture_started = false;
                    continue;
                }
                
                // Check for keyboard keys (only for keyboard keybind targets)
                match current_target {
                    CaptureTarget::MeleeKey | CaptureTarget::JumpKey | CaptureTarget::EmoteKey | CaptureTarget::RapidClickKey => {
                        // Find newly pressed keys (keys that are pressed now but weren't before)
                        for key in &keys {
                            if !last_keys.contains(key) {
                                // Ignore modifier keys and function keys that might be used by the system
                                use device_query::Keycode;
                                match key {
                                    Keycode::LControl | Keycode::RControl |
                                    Keycode::LShift | Keycode::RShift |
                                    Keycode::LAlt | Keycode::RAlt => {
                                        // Skip modifier keys, continue to next key
                                        continue;
                                    }
                                    _ => {
                                        // Found a newly pressed key
                                        let key_name = SharedConfig::keycode_to_string(*key);
                                        eprintln!("Captured key: {} -> {}", format!("{:?}", key), key_name);
                                        
                                        let mut config = config_clone.lock().unwrap();
                                        match current_target {
                                            CaptureTarget::MeleeKey => config.melee_key = key_name.clone(),
                                            CaptureTarget::JumpKey => config.jump_key = key_name.clone(),
                                            CaptureTarget::EmoteKey => config.emote_key = key_name.clone(),
                                            CaptureTarget::RapidClickKey => config.rapid_click_key = key_name.clone(),
                                            _ => {}
                                        }
                                        eprintln!("Updated {} to: {}", 
                                            match current_target {
                                                CaptureTarget::MeleeKey => "melee_key",
                                                CaptureTarget::JumpKey => "jump_key",
                                                CaptureTarget::EmoteKey => "emote_key",
                                                CaptureTarget::RapidClickKey => "rapid_click_key",
                                                _ => "unknown",
                                            },
                                            key_name
                                        );
                                        *capture_target_clone.lock().unwrap() = CaptureTarget::None;
                                        capture_started = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                
                // Check for mouse buttons (only for mouse button targets)
                match current_target {
                    CaptureTarget::AimButton | CaptureTarget::FireButton | CaptureTarget::MacroButton | CaptureTarget::MacroAltButton => {
                        // Find newly pressed mouse buttons
                        // Check all button indices - side buttons can be at various indices
                        eprintln!("Checking mouse buttons - Current: {:?}, Last: {:?}", 
                            mouse.button_pressed.iter().enumerate()
                                .filter(|(_, &p)| p)
                                .map(|(i, _)| i)
                                .collect::<Vec<_>>(),
                            last_mouse_buttons.iter().enumerate()
                                .filter(|(_, p)| **p)
                                .map(|(i, _)| i)
                                .collect::<Vec<_>>()
                        );
                        
                        // Check all possible button indices
                        // device_query's button_pressed is a Vec<bool> where indices might not match pynput exactly
                        // pynput uses: button8 (index 8) and button9 (index 9) for side buttons
                        // But device_query might use different indices, so we check all
                        // Also check beyond the array length in case device_query uses sparse arrays
                        let max_check = mouse.button_pressed.len().max(10); // Check at least up to index 9
                        for idx in 0..max_check {
                            let pressed = mouse.button_pressed.get(idx).copied().unwrap_or(false);
                            if pressed {
                                // Check if this button was just pressed (wasn't pressed before)
                                let was_pressed = last_mouse_buttons.get(idx).copied().unwrap_or(false);
                                if !was_pressed {
                                    eprintln!("✅ Captured mouse button at index: {} (button_pressed.len() = {})", 
                                        idx, mouse.button_pressed.len());
                                    eprintln!("   Full button_pressed array: {:?}", mouse.button_pressed);
                                    
                                    let mut config = config_clone.lock().unwrap();
                                    match current_target {
                                        CaptureTarget::AimButton => {
                                            config.aim_button = idx;
                                            eprintln!("Updated aim_button to: {}", idx);
                                        }
                                        CaptureTarget::FireButton => {
                                            config.fire_button = idx;
                                            eprintln!("Updated fire_button to: {}", idx);
                                        }
                                        CaptureTarget::MacroButton => {
                                            config.macro_button = idx;
                                            eprintln!("Updated macro_button to: {} (this should be 8 for side button 1)", idx);
                                        }
                                        CaptureTarget::MacroAltButton => {
                                            config.macro_alt_button = idx;
                                            eprintln!("Updated macro_alt_button to: {} (this should be 9 for side button 2)", idx);
                                        }
                                        _ => {}
                                    }
                                    *capture_target_clone.lock().unwrap() = CaptureTarget::None;
                                    capture_started = false;
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                
                // Update last state
                last_keys = keys.iter().cloned().collect();
                last_mouse_buttons = mouse.button_pressed.clone();
            }
        });
        
        Self {
            config,
            capture_target,
        }
    }
    
    fn keybind_button(ui: &mut egui::Ui, label: &str, value: &mut String, capture_target: CaptureTarget, capture_target_arc: &Arc<Mutex<CaptureTarget>>) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            
            let current_capture = *capture_target_arc.lock().unwrap();
            let button_text = if current_capture == capture_target {
                "Press key..."
            } else {
                "Set"
            };
            
            let button_color = if current_capture == capture_target {
                egui::Color32::from_rgb(255, 200, 0) // Orange/yellow when listening
            } else {
                egui::Color32::from_rgb(100, 100, 100) // Gray when not listening
            };
            
            let response = ui.add_sized(
                [80.0, 20.0],
                egui::Button::new(button_text).fill(button_color)
            );
            
            if response.clicked() {
                *capture_target_arc.lock().unwrap() = capture_target;
            }
            
            // Show current value
            ui.label(format!("({})", value));
        });
    }
    
    fn mouse_button_slider(ui: &mut egui::Ui, label: &str, value: &mut usize, capture_target: CaptureTarget, capture_target_arc: &Arc<Mutex<CaptureTarget>>) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            
            let current_capture = *capture_target_arc.lock().unwrap();
            let button_text = if current_capture == capture_target {
                "Click button..."
            } else {
                "Set"
            };
            
            let button_color = if current_capture == capture_target {
                egui::Color32::from_rgb(255, 200, 0) // Orange/yellow when listening
            } else {
                egui::Color32::from_rgb(100, 100, 100) // Gray when not listening
            };
            
            let response = ui.add_sized(
                [100.0, 20.0],
                egui::Button::new(button_text).fill(button_color)
            );
            
            if response.clicked() {
                *capture_target_arc.lock().unwrap() = capture_target;
            }
            
            // Show slider for manual adjustment
            ui.add(egui::Slider::new(value, 1..=10));
        });
    }
}

impl eframe::App for MacroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Also check egui's input for keyboard keys (works when window has focus)
        let current_capture = *self.capture_target.lock().unwrap();
        
        // Use egui input for keyboard keys when capturing (works when window has focus)
        match current_capture {
            CaptureTarget::MeleeKey | CaptureTarget::JumpKey | CaptureTarget::EmoteKey | CaptureTarget::RapidClickKey => {
                // Check for key presses via egui events
                let mut captured_key: Option<String> = None;
                ctx.input(|i| {
                    for event in &i.events {
                        if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                            // Skip if modifier keys are held (unless it's just the key itself)
                            if modifiers.ctrl || modifiers.alt || modifiers.shift || modifiers.mac_cmd {
                                continue;
                            }
                            
                            // Convert egui::Key to string
                            let key_name = match key {
                                egui::Key::Space => "Space".to_string(),
                                egui::Key::Enter => "Enter".to_string(),
                                egui::Key::Tab => "Tab".to_string(),
                                egui::Key::Backspace => "Backspace".to_string(),
                                egui::Key::Escape => {
                                    // Cancel capture on Escape
                                    *self.capture_target.lock().unwrap() = CaptureTarget::None;
                                    return;
                                }
                                egui::Key::ArrowUp => "ArrowUp".to_string(),
                                egui::Key::ArrowDown => "ArrowDown".to_string(),
                                egui::Key::ArrowLeft => "ArrowLeft".to_string(),
                                egui::Key::ArrowRight => "ArrowRight".to_string(),
                                egui::Key::F1 => "F1".to_string(),
                                egui::Key::F2 => "F2".to_string(),
                                egui::Key::F3 => "F3".to_string(),
                                egui::Key::F4 => "F4".to_string(),
                                egui::Key::F5 => "F5".to_string(),
                                egui::Key::F6 => "F6".to_string(),
                                egui::Key::F7 => "F7".to_string(),
                                egui::Key::F8 => "F8".to_string(),
                                egui::Key::F9 => "F9".to_string(),
                                egui::Key::F10 => "F10".to_string(),
                                egui::Key::F11 => "F11".to_string(),
                                egui::Key::F12 => "F12".to_string(),
                                _ => {
                                    // For letter keys, try to get the character from text events
                                    let key_str = format!("{:?}", key);
                                    if let Some(letter) = key_str.strip_prefix("Key") {
                                        letter.to_string()
                                    } else {
                                        key_str
                                    }
                                }
                            };
                            
                            eprintln!("Captured key via egui: {:?} -> {}", key, key_name);
                            captured_key = Some(key_name);
                            break;
                        }
                    }
                });
                
                if let Some(key_name) = captured_key {
                    let mut config = self.config.lock().unwrap();
                    match current_capture {
                        CaptureTarget::MeleeKey => config.melee_key = key_name.clone(),
                        CaptureTarget::JumpKey => config.jump_key = key_name.clone(),
                        CaptureTarget::EmoteKey => config.emote_key = key_name.clone(),
                        CaptureTarget::RapidClickKey => config.rapid_click_key = key_name.clone(),
                        _ => {}
                    }
                    eprintln!("Updated config via egui input");
                    *self.capture_target.lock().unwrap() = CaptureTarget::None;
                }
            }
            _ => {}
        }
        
        // Request frequent repaints if we're capturing to show visual feedback
        if current_capture != CaptureTarget::None {
            ctx.request_repaint_after(std::time::Duration::from_millis(50)); // Update UI every 50ms
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Exodia Contagion Macro - Configuration");
            
            ui.separator();
            
            let mut config = self.config.lock().unwrap();
            
            // Timing Configuration
            ui.collapsing("⏱️ Timing Settings", |ui| {
                ui.add(egui::Slider::new(&mut config.fps, 30.0..=300.0).text("Game FPS"));
                ui.add(egui::Slider::new(&mut config.jump_delay_ms, 500.0..=2000.0).text("Jump Delay (ms)"));
                ui.add(egui::Slider::new(&mut config.aim_melee_delay_ms, 1..=100).text("Aim-Melee Delay (ms)"));
                ui.add(egui::Slider::new(&mut config.melee_hold_time_ms, 1..=200).text("Melee Hold Time (ms)"));
                
                ui.checkbox(&mut config.use_emote_formula, "Use Emote Formula (auto-calculated)");
                if !config.use_emote_formula {
                    ui.add(egui::Slider::new(&mut config.emote_preparation_delay_manual_ms, 1..=500).text("Emote Preparation Delay (ms)"));
                }
                
                ui.add(egui::Slider::new(&mut config.rapid_fire_duration_ms, 50..=500).text("Rapid Fire Duration (ms)"));
                ui.add(egui::Slider::new(&mut config.rapid_fire_click_delay_ms, 1..=10).text("Rapid Fire Click Delay (ms)"));
                ui.add(egui::Slider::new(&mut config.sequence_end_delay_ms, 1..=200).text("Sequence End Delay (ms)"));
                ui.add(egui::Slider::new(&mut config.loop_delay_ms, 1..=10).text("Loop Delay (ms)"));
                ui.add(egui::Slider::new(&mut config.rapid_click_count, 1..=50).text("Rapid Click Count"));
                ui.add(egui::Slider::new(&mut config.rapid_click_delay_ms, 1..=200).text("Rapid Click Delay (ms)"));
            });
            
            ui.separator();
            
            // Keybind Configuration
            ui.collapsing("⌨️ Keybind Settings", |ui| {
                // Keyboard keys
                Self::keybind_button(ui, "Melee Key", &mut config.melee_key, CaptureTarget::MeleeKey, &self.capture_target);
                Self::keybind_button(ui, "Jump Key", &mut config.jump_key, CaptureTarget::JumpKey, &self.capture_target);
                Self::keybind_button(ui, "Emote Key", &mut config.emote_key, CaptureTarget::EmoteKey, &self.capture_target);
                Self::keybind_button(ui, "Rapid Click Key", &mut config.rapid_click_key, CaptureTarget::RapidClickKey, &self.capture_target);
                
                ui.separator();
                
                // Mouse buttons
                Self::mouse_button_slider(ui, "Aim Button", &mut config.aim_button, CaptureTarget::AimButton, &self.capture_target);
                Self::mouse_button_slider(ui, "Fire Button", &mut config.fire_button, CaptureTarget::FireButton, &self.capture_target);
                Self::mouse_button_slider(ui, "Macro Button", &mut config.macro_button, CaptureTarget::MacroButton, &self.capture_target);
                
                ui.checkbox(&mut config.enable_macro_alt, "Enable Alternative Macro Button");
                if config.enable_macro_alt {
                    Self::mouse_button_slider(ui, "Alt Macro Button", &mut config.macro_alt_button, CaptureTarget::MacroAltButton, &self.capture_target);
                }
            });
            
            ui.separator();
            
            // Status and info
            let current_capture = *self.capture_target.lock().unwrap();
            if current_capture != CaptureTarget::None {
                ui.label(egui::RichText::new("🎯 Listening for input... Press any key or mouse button (Escape to cancel)").color(egui::Color32::YELLOW));
                #[cfg(target_os = "linux")]
                ui.label(egui::RichText::new("💡 On Linux, side buttons are detected via evdev").color(egui::Color32::LIGHT_BLUE));
            } else {
                ui.label(egui::RichText::new("💡 Changes apply immediately to new macro sequences").color(egui::Color32::GREEN));
            }
            ui.label("Note: Close this window to exit the macro");
        });
    }
}

pub fn run_gui(config: std::sync::Arc<std::sync::Mutex<SharedConfig>>) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 650.0])
            .with_title("Exodia Contagion Macro - Configuration")
            .with_visible(true),
        ..Default::default()
    };
    
    eprintln!("Initializing GUI window...");
    eframe::run_native(
        "Exodia Contagion Macro",
        options,
        Box::new(|_cc| {
            eprintln!("GUI window created successfully");
            Box::new(MacroApp::new(config))
        }),
    )
}
