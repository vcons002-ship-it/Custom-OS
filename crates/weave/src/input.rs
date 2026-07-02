//! Keyboard input for the Weave: raw evdev, no display server in between.
//! A reader thread per keyboard-capable device feeds decoded events into one
//! channel. US layout, enough keys for the Intent Bar.

use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum KeyEvent {
    Char(char),
    Backspace,
    Enter,
}

/// Spawn readers for every device that looks like a keyboard. Returns how
/// many were found (0 in headless CI — the Weave still runs).
pub fn spawn_keyboards(tx: Sender<KeyEvent>) -> usize {
    let mut found = 0;
    for (_path, device) in evdev::enumerate() {
        let is_keyboard = device
            .supported_keys()
            .map(|k| k.contains(evdev::KeyCode::KEY_A) && k.contains(evdev::KeyCode::KEY_ENTER))
            .unwrap_or(false);
        if !is_keyboard {
            continue;
        }
        found += 1;
        let tx = tx.clone();
        std::thread::spawn(move || read_loop(device, tx));
    }
    found
}

fn read_loop(mut device: evdev::Device, tx: Sender<KeyEvent>) {
    let mut shift = false;
    loop {
        let events = match device.fetch_events() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        for ev in events {
            if let evdev::EventSummary::Key(_, key, value) = ev.destructure() {
                let pressed = value != 0; // 1 press, 2 autorepeat
                if key == evdev::KeyCode::KEY_LEFTSHIFT || key == evdev::KeyCode::KEY_RIGHTSHIFT {
                    shift = pressed;
                    continue;
                }
                if !pressed {
                    continue;
                }
                let out = match key {
                    evdev::KeyCode::KEY_ENTER | evdev::KeyCode::KEY_KPENTER => {
                        Some(KeyEvent::Enter)
                    }
                    evdev::KeyCode::KEY_BACKSPACE => Some(KeyEvent::Backspace),
                    k => decode(k, shift).map(KeyEvent::Char),
                };
                if let Some(out) = out {
                    if tx.send(out).is_err() {
                        return;
                    }
                }
            }
        }
    }
}

fn decode(key: evdev::KeyCode, shift: bool) -> Option<char> {
    use evdev::KeyCode as K;
    let (lower, upper) = match key {
        K::KEY_A => ('a', 'A'),
        K::KEY_B => ('b', 'B'),
        K::KEY_C => ('c', 'C'),
        K::KEY_D => ('d', 'D'),
        K::KEY_E => ('e', 'E'),
        K::KEY_F => ('f', 'F'),
        K::KEY_G => ('g', 'G'),
        K::KEY_H => ('h', 'H'),
        K::KEY_I => ('i', 'I'),
        K::KEY_J => ('j', 'J'),
        K::KEY_K => ('k', 'K'),
        K::KEY_L => ('l', 'L'),
        K::KEY_M => ('m', 'M'),
        K::KEY_N => ('n', 'N'),
        K::KEY_O => ('o', 'O'),
        K::KEY_P => ('p', 'P'),
        K::KEY_Q => ('q', 'Q'),
        K::KEY_R => ('r', 'R'),
        K::KEY_S => ('s', 'S'),
        K::KEY_T => ('t', 'T'),
        K::KEY_U => ('u', 'U'),
        K::KEY_V => ('v', 'V'),
        K::KEY_W => ('w', 'W'),
        K::KEY_X => ('x', 'X'),
        K::KEY_Y => ('y', 'Y'),
        K::KEY_Z => ('z', 'Z'),
        K::KEY_1 => ('1', '!'),
        K::KEY_2 => ('2', '@'),
        K::KEY_3 => ('3', '#'),
        K::KEY_4 => ('4', '$'),
        K::KEY_5 => ('5', '%'),
        K::KEY_6 => ('6', '^'),
        K::KEY_7 => ('7', '&'),
        K::KEY_8 => ('8', '*'),
        K::KEY_9 => ('9', '('),
        K::KEY_0 => ('0', ')'),
        K::KEY_SPACE => (' ', ' '),
        K::KEY_MINUS => ('-', '_'),
        K::KEY_EQUAL => ('=', '+'),
        K::KEY_COMMA => (',', '<'),
        K::KEY_DOT => ('.', '>'),
        K::KEY_SLASH => ('/', '?'),
        K::KEY_SEMICOLON => (';', ':'),
        K::KEY_APOSTROPHE => ('\'', '"'),
        _ => return None,
    };
    Some(if shift { upper } else { lower })
}
