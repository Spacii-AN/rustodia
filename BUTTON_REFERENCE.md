# Button and Key Reference Guide

This file contains all available buttons and keys you can use in the macro configuration.

## Mouse Buttons

### Standard Mouse Buttons
```python
Button.left      # Left mouse button
Button.right     # Right mouse button
Button.middle    # Middle mouse button (scroll wheel click)
```

### Side Mouse Buttons (Most Common for Macros)
```python
Button.button8   # First side mouse button (x1) - Recommended for macro
Button.button9   # Second side mouse button (x2) - Alternative macro button
```

### Additional Mouse Buttons
```python
Button.button10  # Additional mouse button
Button.button11  # Additional mouse button
Button.button12  # Additional mouse button
Button.button13  # Additional mouse button
Button.button14  # Additional mouse button
Button.button15  # Additional mouse button
# ... up to button30
```

### Scroll Wheel
```python
Button.scroll_up      # Scroll wheel up
Button.scroll_down    # Scroll wheel down
Button.scroll_left    # Scroll wheel left (if supported)
Button.scroll_right   # Scroll wheel right (if supported)
```

### Other
```python
Button.unknown   # Unknown/unsupported button
```

## Keyboard Keys

### Special Keys
```python
Key.space        # Spacebar
Key.enter        # Enter/Return
Key.tab          # Tab
Key.backspace    # Backspace
Key.delete       # Delete
Key.esc          # Escape
Key.shift        # Left Shift
Key.shift_r      # Right Shift
Key.ctrl         # Left Control
Key.ctrl_r       # Right Control
Key.alt          # Left Alt
Key.alt_r        # Right Alt
Key.alt_gr       # Alt Gr
Key.cmd          # Command (macOS) / Windows key
Key.cmd_r        # Right Command key
```

### Function Keys
```python
Key.f1           # F1
Key.f2           # F2
Key.f3           # F3
Key.f4           # F4
Key.f5           # F5
Key.f6           # F6
Key.f7           # F7
Key.f8           # F8
Key.f9           # F9
Key.f10          # F10
Key.f11          # F11 (used for macro toggle)
Key.f12          # F12
Key.f13           # F13 (macOS)
Key.f14           # F14 (macOS)
Key.f15           # F15 (macOS)
Key.f16           # F16 (macOS)
Key.f17           # F17 (macOS)
Key.f18           # F18 (macOS)
Key.f19           # F19 (macOS)
Key.f20           # F20 (macOS)
```

### Arrow Keys
```python
Key.up           # Up arrow
Key.down         # Down arrow
Key.left         # Left arrow
Key.right        # Right arrow
```

### Navigation Keys
```python
Key.home         # Home
Key.end          # End
Key.page_up      # Page Up
Key.page_down    # Page Down
```

### Media Keys
```python
Key.media_play_pause    # Play/Pause
Key.media_next          # Next track
Key.media_previous      # Previous track
Key.media_volume_up     # Volume up
Key.media_volume_down   # Volume down
Key.media_volume_mute   # Mute
```

### Other Keys
```python
Key.caps_lock    # Caps Lock
Key.num_lock     # Num Lock
Key.scroll_lock  # Scroll Lock
Key.print_screen # Print Screen
Key.pause        # Pause/Break
Key.insert       # Insert
Key.menu         # Menu/Context menu key
```

## Regular Character Keys

For regular letter and number keys, use the character as a string:

```python
'a'              # Letter a
'b'              # Letter b
# ... all letters a-z

'1'              # Number 1
'2'              # Number 2
# ... all numbers 0-9

'.'              # Period
','              # Comma
';'              # Semicolon
'/'              # Forward slash
'\'              # Backslash
'['              # Left bracket
']'              # Right bracket
'-'              # Hyphen
'='              # Equals
# ... and other printable characters
```

## Usage Examples

### In KEYBINDS dictionary:

```python
KEYBINDS = {
    'melee': 'e',                    # Regular letter key
    'jump': Key.space,               # Special key
    'aim': Button.right,             # Mouse button
    'fire': Button.left,             # Mouse button
    'emote': '.',                    # Character key
    'macro': Button.button8,          # Side mouse button
    'macro_alt': Button.button9,     # Alternative side button
    'rapid_click': 'j',              # Regular letter key
}
```

### Common Alternatives for Macro Button:

```python
# Mouse buttons
'macro': Button.button8,      # First side button (most common)
'macro': Button.button9,      # Second side button
'macro': Button.middle,       # Middle mouse button
'macro': Button.right,        # Right mouse button (not recommended - conflicts with aim)

# Keyboard keys
'macro': Key.f1,              # Function key
'macro': Key.f2,              # Function key
'macro': Key.ctrl,            # Control key
'macro': Key.shift,           # Shift key
'macro': 'q',                 # Letter key
'macro': 'z',                 # Letter key
```

## Notes

- **Side mouse buttons** (`Button.button8` and `Button.button9`) are the most common choice for macros as they don't interfere with normal gameplay
- **Function keys** (F1-F12) are also good choices as they're easily accessible and rarely used in games
- **Regular letter keys** can be used, but make sure they don't conflict with your game controls
- **Mouse buttons** like `Button.left` and `Button.right` are not recommended for macros as they're used for normal gameplay actions

