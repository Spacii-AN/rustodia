mod config;
mod gui;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::process;
use std::env;

use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, Settings, Keyboard, Mouse};
use sysinfo::System;

use config::SharedConfig;

// ============================================================================
// KEYBIND CONFIGURATION
// ============================================================================
// Edit the values below to customize your keybinds
// ============================================================================

// Keybind Settings (internal structure, converted from SharedConfig)
#[derive(Clone, Copy)]
pub struct Keybinds {
    pub melee: Keycode,
    pub jump: Keycode,
    pub aim: usize, // Mouse button index
    pub fire: usize, // Mouse button index
    pub emote: Keycode,
    pub macro_button: usize, // Mouse button index
    pub macro_alt: Option<usize>, // Mouse button index
    pub rapid_click: Keycode,
}

// ============================================================================
// TIMING CONFIGURATION
// ============================================================================
// Adjust these values to fine-tune the macro timing
// ============================================================================

// ============================================================================
// END OF USER CONFIGURATION
// ============================================================================

// Global state
struct State {
    running: AtomicBool,
    macro_enabled: AtomicBool,
    warframe_active: AtomicBool,
    rapid_clicking: AtomicBool,
}

impl State {
    fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            macro_enabled: AtomicBool::new(true),
            warframe_active: AtomicBool::new(false),
            rapid_clicking: AtomicBool::new(false),
        }
    }
}

// High-precision sleep using busy-wait for short durations
#[inline(always)]
fn precise_sleep(duration: Duration) {
    if duration.is_zero() {
        return;
    }
    
    let start = Instant::now();
    let target = start + duration;
    
    // For longer sleeps, use thread::sleep with compensation
    if duration.as_millis() > 40 {
        let compensation = duration.saturating_sub(Duration::from_millis(20));
        if !compensation.is_zero() {
            thread::sleep(compensation);
        }
    }
    
    // Busy-wait for remaining time (high precision)
    while Instant::now() < target {
        std::hint::spin_loop();
    }
}

// Cached system for window detection (thread-local to avoid synchronization overhead)
thread_local! {
    static SYSTEM_CACHE: std::cell::RefCell<Option<System>> = std::cell::RefCell::new(None);
}

// Check if Warframe is the active window
fn is_warframe_active() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // Try xdotool first (faster)
        if let Ok(output) = Command::new("xdotool")
            .arg("getactivewindow")
            .arg("getwindowname")
            .output()
        {
            if let Ok(name) = String::from_utf8(output.stdout) {
                // Use case-insensitive check without allocation
                return name.as_bytes().windows(8).any(|w| w.eq_ignore_ascii_case(b"warframe"));
            }
        }
        
        // Fallback: check process list (cached system)
        SYSTEM_CACHE.with(|sys| {
            let mut system = sys.borrow_mut();
            if system.is_none() {
                *system = Some(System::new());
            }
            if let Some(ref mut s) = *system {
                s.refresh_all();
                for process in s.processes().values() {
                    if let Some(name) = process.name().to_str() {
                        if name.as_bytes().windows(8).any(|w| w.eq_ignore_ascii_case(b"warframe")) {
                            return true;
                        }
                    }
                }
            }
            false
        })
    }
    
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winuser::{GetForegroundWindow, GetWindowThreadProcessId};
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
        use winapi::um::handleapi::CloseHandle;
        use std::ffi::CString;
        use std::os::raw::c_char;
        
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return false;
            }
            
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            
            if pid == 0 {
                return false;
            }
            
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            
            SYSTEM_CACHE.with(|sys| {
                let mut system = sys.borrow_mut();
                if system.is_none() {
                    *system = Some(System::new());
                }
                if let Some(ref mut s) = *system {
                    s.refresh_process(sysinfo::Pid::from_u32(pid));
                    if let Some(process) = s.process(sysinfo::Pid::from_u32(pid)) {
                        let name = process.name();
                        // Use byte comparison for better performance
                        let result = name.as_bytes().windows(8).any(|w| w.eq_ignore_ascii_case(b"warframe"));
                        CloseHandle(handle);
                        return result;
                    }
                }
                CloseHandle(handle);
                false
            })
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of first application process whose frontmost is true")
            .output()
        {
            if let Ok(name) = String::from_utf8(output.stdout) {
                return name.as_bytes().windows(8).any(|w| w.eq_ignore_ascii_case(b"warframe"));
            }
        }
        false
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        true // Default to true for unknown platforms
    }
}

// Set process to high priority
fn set_high_priority() {
    #[cfg(target_os = "linux")]
    {
        use libc::{setpriority, PRIO_PROCESS, getpid};
        unsafe {
            setpriority(PRIO_PROCESS, getpid() as u32, -10);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use winapi::um::processthreadsapi::{SetPriorityClass, GetCurrentProcess, HIGH_PRIORITY_CLASS};
        unsafe {
            let handle = GetCurrentProcess();
            SetPriorityClass(handle, HIGH_PRIORITY_CLASS);
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use libc::{setpriority, PRIO_PROCESS, getpid};
        unsafe {
            setpriority(PRIO_PROCESS, getpid() as u32, 10);
        }
    }
}

// Precomputed key mappings for performance
#[derive(Clone, Copy)]
struct PrecomputedKeys {
    jump: enigo::Key,
    melee: enigo::Key,
    emote: enigo::Key,
    #[allow(dead_code)]
    rapid_click: enigo::Key, // Precomputed but not currently used (rapid click uses keycode directly)
}

impl PrecomputedKeys {
    fn from_keybinds(keybinds: &Keybinds) -> Self {
        Self {
            jump: match keybinds.jump {
                Keycode::Space => enigo::Key::Space,
                _ => enigo::Key::Unicode(' '),
            },
            melee: match keybinds.melee {
                Keycode::E => enigo::Key::Unicode('e'),
                _ => enigo::Key::Unicode(' '),
            },
            emote: match keybinds.emote {
                Keycode::Dot => enigo::Key::Unicode('.'),
                _ => enigo::Key::Unicode(' '),
            },
            rapid_click: match keybinds.rapid_click {
                Keycode::J => enigo::Key::Unicode('j'),
                _ => enigo::Key::Unicode(' '),
            },
        }
    }
}

// Button lookup table for O(1) access
// device_query uses 0-based indexing: 1=Left, 2=Right, 3=Middle, 8=Side1, 9=Side2
// This matches pynput's Button.button8 (index 8) and Button.button9 (index 9)
const BUTTON_LOOKUP: [Option<enigo::Button>; 10] = [
    None,                                    // 0
    Some(enigo::Button::Left),              // 1 = Left
    Some(enigo::Button::Right),             // 2 = Right
    Some(enigo::Button::Middle),            // 3 = Middle
    None,                                    // 4
    None,                                    // 5
    None,                                    // 6
    None,                                    // 7
    Some(enigo::Button::Left),              // 8 = Side button 1 (button8 in pynput) - using Left as fallback
    Some(enigo::Button::Left),              // 9 = Side button 2 (button9 in pynput) - using Left as fallback
];

#[inline(always)]
fn button_from_index(idx: usize) -> enigo::Button {
    // For side buttons (8, 9), we need to use a different approach since enigo might not have direct support
    // We'll use the lookup for standard buttons, and for side buttons we'll need special handling
    if idx == 8 {
        // Side button 1 - try to use a workaround or map to available button
        // Note: enigo might not support side buttons directly, so we may need to use xdotool on Linux
        enigo::Button::Left // Fallback for now
    } else if idx == 9 {
        // Side button 2
        enigo::Button::Left // Fallback for now
    } else {
        BUTTON_LOOKUP.get(idx).and_then(|&b| b).unwrap_or(enigo::Button::Left)
    }
}

// Helper functions to get durations from config
fn get_durations_from_config(config: &SharedConfig) -> (Duration, Duration, Duration, Duration, Duration, Duration, Duration, Duration) {
    (
        config.double_jump_delay(),
        Duration::from_millis(config.aim_melee_delay_ms),
        Duration::from_millis(config.melee_hold_time_ms),
        config.emote_preparation_delay(),
        Duration::from_millis(config.rapid_fire_click_delay_ms),
        Duration::from_millis(config.sequence_end_delay_ms),
        Duration::from_millis(config.loop_delay_ms),
        Duration::from_millis(config.rapid_click_delay_ms),
    )
}

// Execute one complete Exodia Contagion sequence
#[inline]
fn execute_contagion_sequence(
    enigo: &mut Enigo,
    state: &State,
    keys: &PrecomputedKeys,
    aim_button: enigo::Button,
    fire_button: enigo::Button,
    config: &SharedConfig,
) {
    if !state.running.load(Ordering::Relaxed) {
        return;
    }
    
    let (double_jump_delay, aim_melee_delay, melee_hold_time, emote_prep_delay, 
         rapid_fire_click_delay, sequence_end_delay, _, _) = get_durations_from_config(config);
    
    // Double jump
    let _ = enigo.key(keys.jump, enigo::Direction::Press);
    precise_sleep(double_jump_delay);
    let _ = enigo.key(keys.jump, enigo::Direction::Release);
    
    let _ = enigo.key(keys.jump, enigo::Direction::Press);
    precise_sleep(double_jump_delay);
    let _ = enigo.key(keys.jump, enigo::Direction::Release);
    
    // Aim and melee
    let _ = enigo.button(aim_button, enigo::Direction::Press);
    precise_sleep(aim_melee_delay);
    
    let _ = enigo.key(keys.melee, enigo::Direction::Press);
    precise_sleep(melee_hold_time);
    let _ = enigo.key(keys.melee, enigo::Direction::Release);
    
    let _ = enigo.button(aim_button, enigo::Direction::Release);
    
    // Emote cancel
    precise_sleep(emote_prep_delay);
    
    let _ = enigo.key(keys.emote, enigo::Direction::Press);
    precise_sleep(double_jump_delay);
    let _ = enigo.key(keys.emote, enigo::Direction::Release);
    
    let _ = enigo.key(keys.emote, enigo::Direction::Press);
    precise_sleep(double_jump_delay);
    let _ = enigo.key(keys.emote, enigo::Direction::Release);
    
    // Rapid fire - optimized loop
    let start_time = Instant::now();
    let duration_limit = config.rapid_fire_duration_ms as u128;
    
    while state.running.load(Ordering::Relaxed) {
        let _ = enigo.button(fire_button, enigo::Direction::Press);
        let _ = enigo.button(fire_button, enigo::Direction::Release);
        precise_sleep(rapid_fire_click_delay);
        
        // Check elapsed time less frequently for better performance
        if start_time.elapsed().as_millis() > duration_limit {
            break;
        }
    }
    
    // End-of-sequence delay
    if state.running.load(Ordering::Relaxed) {
        precise_sleep(sequence_end_delay);
    }
}

// Main loop that executes contagion sequences while key is held
fn contagion_loop(state: Arc<State>, config: Arc<Mutex<SharedConfig>>) {
    let settings = Settings::default();
    let mut enigo = match Enigo::new(&settings) {
        Ok(e) => e,
        Err(_) => return,
    };
    
    while state.running.load(Ordering::Relaxed) {
        // Get current config snapshot
        let config_snapshot = config.lock().unwrap().clone();
        let keybinds = config_snapshot.to_keybinds();
        
        // Precompute everything once per iteration
        let keys = PrecomputedKeys::from_keybinds(&keybinds);
        let fire_button = button_from_index(keybinds.fire);
        let aim_button = button_from_index(keybinds.aim);
        
        execute_contagion_sequence(&mut enigo, &state, &keys, aim_button, fire_button, &config_snapshot);
        
        let (_, _, _, _, _, _, loop_delay, _) = get_durations_from_config(&config_snapshot);
        precise_sleep(loop_delay);
    }
    
    // Cleanup: release all keys/buttons
    let config_snapshot = config.lock().unwrap().clone();
    let keybinds = config_snapshot.to_keybinds();
    let keys = PrecomputedKeys::from_keybinds(&keybinds);
    let fire_button = button_from_index(keybinds.fire);
    let aim_button = button_from_index(keybinds.aim);
    
    let _ = enigo.key(keys.melee, enigo::Direction::Release);
    let _ = enigo.key(keys.emote, enigo::Direction::Release);
    let _ = enigo.button(aim_button, enigo::Direction::Release);
    let _ = enigo.button(fire_button, enigo::Direction::Release);
}

// Execute rapid click sequence
fn execute_rapid_click(state: Arc<State>, config: Arc<Mutex<SharedConfig>>) {
    state.rapid_clicking.store(true, Ordering::Relaxed);
    let settings = Settings::default();
    let mut enigo = match Enigo::new(&settings) {
        Ok(e) => e,
        Err(_) => {
            state.rapid_clicking.store(false, Ordering::Relaxed);
            return;
        }
    };
    
    let config_snapshot = config.lock().unwrap().clone();
    let keybinds = config_snapshot.to_keybinds();
    let fire_button = button_from_index(keybinds.fire);
    let rapid_click_delay = Duration::from_millis(config_snapshot.rapid_click_delay_ms);
    
    for _ in 0..config_snapshot.rapid_click_count {
        if !state.macro_enabled.load(Ordering::Relaxed) {
            break;
        }
        
        let _ = enigo.button(fire_button, enigo::Direction::Press);
        let _ = enigo.button(fire_button, enigo::Direction::Release);
        precise_sleep(rapid_click_delay);
    }
    
    state.rapid_clicking.store(false, Ordering::Relaxed);
}

// Background thread to monitor Warframe window state
fn background_app_check(state: Arc<State>) {
    let mut last_state = false;
    loop {
        let current_state = is_warframe_active();
        
        // Only update and print if state changed
        if current_state != last_state {
            state.warframe_active.store(current_state, Ordering::Relaxed);
            if !current_state && state.running.load(Ordering::Relaxed) {
                state.running.store(false, Ordering::Relaxed);
                println!("Warframe window lost focus - macro stopped");
            }
            last_state = current_state;
        } else {
            // If state unchanged, just update atomic (cheaper)
            state.warframe_active.store(current_state, Ordering::Relaxed);
        }
        
        thread::sleep(Duration::from_secs(1));
    }
}

fn run_macro(config: Arc<Mutex<SharedConfig>>, state: Arc<State>) {
    println!("=== Exodia Contagion Macro for Warframe (Rust - Optimized) ===");
    let config_snapshot = config.lock().unwrap().clone();
    println!("\nKEY SETTINGS:");
    println!("  - Hold side mouse button to activate the contagion sequence");
    println!("  - Press '{}' to perform {} rapid clicks", config_snapshot.rapid_click_key, config_snapshot.rapid_click_count);
    println!("  - Press F11 to toggle all macros on/off");
    println!("\nPress Ctrl+C to exit\n");
    
    // Start background window monitoring
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        background_app_check(state_clone);
    });
    
    // Input monitoring loop
    let state_input = Arc::clone(&state);
    let config_input = Arc::clone(&config);
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut last_macro_state = false;
        let mut last_rapid_click_state = false;
        
        loop {
            let keys = device_state.get_keys();
            let mouse = device_state.get_mouse();
            
            // Get current config
            let config_snapshot = config_input.lock().unwrap().clone();
            let keybinds = config_snapshot.to_keybinds();
            
            // Check for F11 toggle
            if keys.contains(&Keycode::F11) {
                let current = state_input.macro_enabled.load(Ordering::Relaxed);
                state_input.macro_enabled.store(!current, Ordering::Relaxed);
                println!("Macro {}", if !current { "enabled" } else { "disabled" });
                thread::sleep(Duration::from_millis(200)); // Debounce
            }
            
            // Only process macro inputs if Warframe is active (to avoid interfering with GUI)
            let warframe_active = state_input.warframe_active.load(Ordering::Relaxed);
            
            if warframe_active {
                // Check for rapid click key
                let rapid_click_pressed = keys.contains(&keybinds.rapid_click);
                if rapid_click_pressed && !last_rapid_click_state 
                    && state_input.macro_enabled.load(Ordering::Relaxed) {
                    let state_clone = Arc::clone(&state_input);
                    let config_clone = Arc::clone(&config_input);
                    thread::spawn(move || {
                        execute_rapid_click(state_clone, config_clone);
                    });
                }
                last_rapid_click_state = rapid_click_pressed;
                
                // Check for macro button
                let macro_button_idx = keybinds.macro_button;
                let macro_pressed = macro_button_idx < mouse.button_pressed.len() 
                    && mouse.button_pressed[macro_button_idx]
                    || (keybinds.macro_alt.is_some() 
                        && {
                            let alt_idx = keybinds.macro_alt.unwrap();
                            alt_idx < mouse.button_pressed.len() && mouse.button_pressed[alt_idx]
                        });
                
                if macro_pressed && !last_macro_state 
                    && !state_input.running.load(Ordering::Relaxed)
                    && state_input.macro_enabled.load(Ordering::Relaxed) {
                    state_input.running.store(true, Ordering::Relaxed);
                    let state_clone = Arc::clone(&state_input);
                    let config_clone = Arc::clone(&config_input);
                    thread::spawn(move || {
                        contagion_loop(state_clone, config_clone);
                    });
                } else if !macro_pressed && last_macro_state {
                    state_input.running.store(false, Ordering::Relaxed);
                }
                last_macro_state = macro_pressed;
            } else {
                // Reset states when Warframe is not active to avoid stuck states
                last_rapid_click_state = false;
                last_macro_state = false;
            }
            
            // Adaptive polling: faster when active, slower when idle
            let sleep_duration = if state_input.running.load(Ordering::Relaxed) {
                Duration::from_micros(500) // 0.5ms when macro is running
            } else {
                Duration::from_millis(2) // 2ms when idle
            };
            thread::sleep(sleep_duration);
        }
    });
    
    // Wait for exit signal
    ctrlc::set_handler(move || {
        println!("\n\nShutting down macro...");
        println!("Goodbye!");
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");
    
    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    set_high_priority();
    
    // Check for GUI mode
    let args: Vec<String> = env::args().collect();
    let use_gui = args.iter().any(|arg| arg == "--gui" || arg == "-g");
    
    let config = Arc::new(Mutex::new(SharedConfig::default()));
    let state = Arc::new(State::new());
    
    if use_gui {
        println!("Starting GUI mode...");
        println!("Note: Keybind configuration works independently - Warframe does not need to be open");
        
        // Start macro in background (but it will only activate when Warframe is open)
        let config_macro = Arc::clone(&config);
        let state_macro = Arc::clone(&state);
        
        thread::spawn(move || {
            run_macro(config_macro, state_macro);
        });
        
        // Small delay to let macro thread start
        thread::sleep(Duration::from_millis(100));
        
        // Run GUI (blocks until window closed)
        // The GUI has its own independent key capture thread that works regardless of Warframe
        println!("Opening GUI window...");
        match gui::run_gui(config) {
            Ok(()) => println!("GUI closed normally"),
            Err(e) => {
                eprintln!("GUI Error: {}", e);
                eprintln!("Falling back to CLI mode...");
                // Keep the macro running even if GUI fails
                loop {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    } else {
        // CLI mode - just run the macro
        run_macro(config, state);
    }
}

