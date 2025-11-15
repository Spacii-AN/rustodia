use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::process;

use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, Settings, Keyboard, Mouse};
use sysinfo::System;

// ============================================================================
// KEYBIND CONFIGURATION
// ============================================================================
// Edit the values below to customize your keybinds
// ============================================================================

// Enable/disable alternative macro button (second side mouse button)
const ENABLE_MACRO_ALT: bool = true;

// Keybind Settings
// For mouse buttons: MouseButton::Left, MouseButton::Right, MouseButton::Middle, MouseButton::Side, MouseButton::Extra
// For keyboard: Keycode::Space, Keycode::E, Keycode::J, Keycode::Period, Keycode::F11
#[derive(Clone, Copy)]
struct Keybinds {
    melee: Keycode,
    jump: Keycode,
    aim: usize, // Mouse button index
    fire: usize, // Mouse button index
    emote: Keycode,
    macro_button: usize, // Mouse button index
    macro_alt: Option<usize>, // Mouse button index
    rapid_click: Keycode,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            melee: Keycode::E,
            jump: Keycode::Space,
            aim: 2, // Right mouse button
            fire: 1, // Left mouse button
            emote: Keycode::Dot, // Period/dot key
            macro_button: 4, // Side button (x1/button8)
            macro_alt: if ENABLE_MACRO_ALT { Some(5) } else { None }, // Extra button (x2/button9)
            rapid_click: Keycode::J,
        }
    }
}

// ============================================================================
// TIMING CONFIGURATION
// ============================================================================
// Adjust these values to fine-tune the macro timing
// ============================================================================

// --- Game FPS ---
const FPS: f64 = 160.0;

// --- Jump Timing ---
const JUMP_DELAY_MS: f64 = 1100.0;
fn double_jump_delay() -> Duration {
    Duration::from_secs_f64(JUMP_DELAY_MS / FPS / 1000.0)
}

// --- Aim & Melee Timing ---
const AIM_MELEE_DELAY_MS: u64 = 25;
const MELEE_HOLD_TIME_MS: u64 = 50;
fn aim_melee_delay() -> Duration { Duration::from_millis(AIM_MELEE_DELAY_MS) }
fn melee_hold_time() -> Duration { Duration::from_millis(MELEE_HOLD_TIME_MS) }

// --- Emote Cancel Timing ---
// Formula-based: -26 * ln(fps) + 245 (automatically calculated)
const USE_EMOTE_FORMULA: bool = true;
const EMOTE_PREPARATION_DELAY_MANUAL_MS: u64 = 100;
fn emote_preparation_delay_manual() -> Duration { Duration::from_millis(EMOTE_PREPARATION_DELAY_MANUAL_MS) }

fn emote_preparation_delay() -> Duration {
    if USE_EMOTE_FORMULA {
        let raw_delay_ms = (-26.0 * FPS.ln() + 245.0).max(0.0) as u64;
        Duration::from_millis(raw_delay_ms)
    } else {
        emote_preparation_delay_manual()
    }
}

// --- Rapid Fire Timing ---
const RAPID_FIRE_DURATION_MS: u64 = 230;
const RAPID_FIRE_CLICK_DELAY_MS: u64 = 1;
fn rapid_fire_click_delay() -> Duration { Duration::from_millis(RAPID_FIRE_CLICK_DELAY_MS) }

// --- Sequence Loop Timing ---
const SEQUENCE_END_DELAY_MS: u64 = 50;
const LOOP_DELAY_MS: u64 = 1;
fn sequence_end_delay() -> Duration { Duration::from_millis(SEQUENCE_END_DELAY_MS) }
fn loop_delay() -> Duration { Duration::from_millis(LOOP_DELAY_MS) }

// --- Rapid Click Macro Timing ---
const RAPID_CLICK_COUNT: usize = 10;
const RAPID_CLICK_DELAY_MS: u64 = 50;
fn rapid_click_delay() -> Duration { Duration::from_millis(RAPID_CLICK_DELAY_MS) }

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

// Check if Warframe is the active window
fn is_warframe_active() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("xdotool")
            .arg("getactivewindow")
            .arg("getwindowname")
            .output()
        {
            if let Ok(name) = String::from_utf8(output.stdout) {
                return name.to_lowercase().contains("warframe");
            }
        }
        
        // Fallback: check process list
        let mut system = System::new();
        system.refresh_all();
        for process in system.processes().values() {
            if let Some(name) = process.name().to_str() {
                if name.to_ascii_lowercase().contains("warframe") {
                    return true;
                }
            }
        }
        false
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
            
            let mut system = System::new();
            system.refresh_process(sysinfo::Pid::from_u32(pid));
            
            if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
                let name = process.name().to_ascii_lowercase();
                CloseHandle(handle);
                return name.contains("warframe");
            }
            
            CloseHandle(handle);
            false
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
                return name.to_lowercase().contains("warframe");
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

// Convert Keycode to enigo key
fn keycode_to_enigo_key(keycode: Keycode) -> enigo::Key {
    match keycode {
        Keycode::Space => enigo::Key::Space,
        Keycode::E => enigo::Key::Unicode('e'),
        Keycode::J => enigo::Key::Unicode('j'),
        Keycode::Dot => enigo::Key::Unicode('.'),
        Keycode::F11 => enigo::Key::F11,
        _ => enigo::Key::Unicode(' '), // Fallback
    }
}

// Execute one complete Exodia Contagion sequence
fn execute_contagion_sequence(
    enigo: &mut Enigo,
    state: &State,
    keybinds: &Keybinds,
) {
    if !state.running.load(Ordering::Relaxed) {
        return;
    }
    
    // Double jump
    enigo.key(keycode_to_enigo_key(keybinds.jump), enigo::Direction::Press);
    precise_sleep(double_jump_delay());
    enigo.key(keycode_to_enigo_key(keybinds.jump), enigo::Direction::Release);
    
    enigo.key(keycode_to_enigo_key(keybinds.jump), enigo::Direction::Press);
    precise_sleep(double_jump_delay());
    enigo.key(keycode_to_enigo_key(keybinds.jump), enigo::Direction::Release);
    
    // Aim and melee
    if keybinds.aim == 2 {
        enigo.button(enigo::Button::Right, enigo::Direction::Press);
    } else if keybinds.aim == 1 {
        enigo.button(enigo::Button::Left, enigo::Direction::Press);
    } else if keybinds.aim == 3 {
        enigo.button(enigo::Button::Middle, enigo::Direction::Press);
    }
    precise_sleep(aim_melee_delay());
    
    enigo.key(keycode_to_enigo_key(keybinds.melee), enigo::Direction::Press);
    precise_sleep(melee_hold_time());
    enigo.key(keycode_to_enigo_key(keybinds.melee), enigo::Direction::Release);
    
    if keybinds.aim == 2 {
        enigo.button(enigo::Button::Right, enigo::Direction::Release);
    } else if keybinds.aim == 1 {
        enigo.button(enigo::Button::Left, enigo::Direction::Release);
    } else if keybinds.aim == 3 {
        enigo.button(enigo::Button::Middle, enigo::Direction::Release);
    }
    
    // Emote cancel
    precise_sleep(emote_preparation_delay());
    
    enigo.key(keycode_to_enigo_key(keybinds.emote), enigo::Direction::Press);
    precise_sleep(double_jump_delay());
    enigo.key(keycode_to_enigo_key(keybinds.emote), enigo::Direction::Release);
    
    enigo.key(keycode_to_enigo_key(keybinds.emote), enigo::Direction::Press);
    precise_sleep(double_jump_delay());
    enigo.key(keycode_to_enigo_key(keybinds.emote), enigo::Direction::Release);
    
    // Rapid fire
    let start_time = Instant::now();
    let fire_button = if keybinds.fire == 1 {
        enigo::Button::Left
    } else if keybinds.fire == 2 {
        enigo::Button::Right
    } else if keybinds.fire == 3 {
        enigo::Button::Middle
    } else {
        enigo::Button::Left
    };
    
    while state.running.load(Ordering::Relaxed) {
        enigo.button(fire_button, enigo::Direction::Press);
        enigo.button(fire_button, enigo::Direction::Release);
        precise_sleep(rapid_fire_click_delay());
        
        let elapsed_ms = start_time.elapsed().as_millis();
        if elapsed_ms > RAPID_FIRE_DURATION_MS as u128 {
            break;
        }
    }
    
    // End-of-sequence delay
    if state.running.load(Ordering::Relaxed) {
        precise_sleep(sequence_end_delay());
    }
}

// Main loop that executes contagion sequences while key is held
fn contagion_loop(state: Arc<State>, keybinds: Keybinds) {
    let settings = Settings::default();
    let mut enigo = match Enigo::new(&settings) {
        Ok(e) => e,
        Err(_) => return,
    };
    
    let fire_button = if keybinds.fire == 1 {
        enigo::Button::Left
    } else if keybinds.fire == 2 {
        enigo::Button::Right
    } else if keybinds.fire == 3 {
        enigo::Button::Middle
    } else {
        enigo::Button::Left
    };
    let aim_button = if keybinds.aim == 1 {
        enigo::Button::Left
    } else if keybinds.aim == 2 {
        enigo::Button::Right
    } else if keybinds.aim == 3 {
        enigo::Button::Middle
    } else {
        enigo::Button::Right
    };
    
    while state.running.load(Ordering::Relaxed) {
        execute_contagion_sequence(&mut enigo, &state, &keybinds);
        precise_sleep(loop_delay());
    }
    
    // Cleanup: release all keys/buttons
    enigo.key(keycode_to_enigo_key(keybinds.melee), enigo::Direction::Release);
    enigo.key(keycode_to_enigo_key(keybinds.emote), enigo::Direction::Release);
    enigo.button(aim_button, enigo::Direction::Release);
    enigo.button(fire_button, enigo::Direction::Release);
}

// Execute rapid click sequence
fn execute_rapid_click(state: Arc<State>, keybinds: Keybinds) {
    state.rapid_clicking.store(true, Ordering::Relaxed);
    let settings = Settings::default();
    let mut enigo = match Enigo::new(&settings) {
        Ok(e) => e,
        Err(_) => {
            state.rapid_clicking.store(false, Ordering::Relaxed);
            return;
        }
    };
    
    let fire_button = if keybinds.fire == 1 {
        enigo::Button::Left
    } else if keybinds.fire == 2 {
        enigo::Button::Right
    } else if keybinds.fire == 3 {
        enigo::Button::Middle
    } else {
        enigo::Button::Left
    };
    
    for _ in 0..RAPID_CLICK_COUNT {
        if !state.macro_enabled.load(Ordering::Relaxed) {
            break;
        }
        
        enigo.button(fire_button, enigo::Direction::Press);
        enigo.button(fire_button, enigo::Direction::Release);
        precise_sleep(rapid_click_delay());
    }
    
    state.rapid_clicking.store(false, Ordering::Relaxed);
}

// Background thread to monitor Warframe window state
fn background_app_check(state: Arc<State>) {
    loop {
        let current_state = is_warframe_active();
        
        if !current_state && state.running.load(Ordering::Relaxed) {
            state.running.store(false, Ordering::Relaxed);
            println!("Warframe window lost focus - macro stopped");
        }
        
        state.warframe_active.store(current_state, Ordering::Relaxed);
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    set_high_priority();
    
    let state = Arc::new(State::new());
    let keybinds = Keybinds::default();
    
    println!("=== Exodia Contagion Macro for Warframe (Rust - Optimized) ===");
    println!("\nKEY SETTINGS:");
    println!("  - Hold side mouse button to activate the contagion sequence");
    println!("  - Press 'j' to perform {} rapid clicks", RAPID_CLICK_COUNT);
    println!("  - Press F11 to toggle all macros on/off");
    println!("\nPress Ctrl+C to exit\n");
    
    // Start background window monitoring
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        background_app_check(state_clone);
    });
    
    // Input monitoring loop
    let state_input = Arc::clone(&state);
    let keybinds_input = keybinds;
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut last_macro_state = false;
        let mut last_rapid_click_state = false;
        
        loop {
            let keys = device_state.get_keys();
            let mouse = device_state.get_mouse();
            
            // Check for F11 toggle
            if keys.contains(&Keycode::F11) {
                let current = state_input.macro_enabled.load(Ordering::Relaxed);
                state_input.macro_enabled.store(!current, Ordering::Relaxed);
                println!("Macro {}", if !current { "enabled" } else { "disabled" });
                thread::sleep(Duration::from_millis(200)); // Debounce
            }
            
            // Check for rapid click key
            let rapid_click_pressed = keys.contains(&keybinds_input.rapid_click);
            if rapid_click_pressed && !last_rapid_click_state 
                && state_input.macro_enabled.load(Ordering::Relaxed)
                && state_input.warframe_active.load(Ordering::Relaxed) {
                let state_clone = Arc::clone(&state_input);
                thread::spawn(move || {
                    execute_rapid_click(state_clone, keybinds_input);
                });
            }
            last_rapid_click_state = rapid_click_pressed;
            
            // Check for macro button
            // button_pressed is a Vec<bool> indexed by button number
            let macro_button_idx = keybinds_input.macro_button;
            let macro_pressed = macro_button_idx < mouse.button_pressed.len() 
                && mouse.button_pressed[macro_button_idx]
                || (keybinds_input.macro_alt.is_some() 
                    && {
                        let alt_idx = keybinds_input.macro_alt.unwrap();
                        alt_idx < mouse.button_pressed.len() && mouse.button_pressed[alt_idx]
                    });
            
            if macro_pressed && !last_macro_state 
                && !state_input.running.load(Ordering::Relaxed)
                && state_input.macro_enabled.load(Ordering::Relaxed)
                && state_input.warframe_active.load(Ordering::Relaxed) {
                state_input.running.store(true, Ordering::Relaxed);
                let state_clone = Arc::clone(&state_input);
                thread::spawn(move || {
                    contagion_loop(state_clone, keybinds_input);
                });
            } else if !macro_pressed && last_macro_state {
                state_input.running.store(false, Ordering::Relaxed);
            }
            last_macro_state = macro_pressed;
            
            thread::sleep(Duration::from_millis(1)); // Poll every 1ms for responsiveness
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

