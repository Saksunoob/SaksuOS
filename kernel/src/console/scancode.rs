#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum KeyCode {
    // --- Standard Set 1 Scan Codes ---
    Escape = 0x01,
    Key1 = 0x02,
    Key2 = 0x03,
    Key3 = 0x04,
    Key4 = 0x05,
    Key5 = 0x06,
    Key6 = 0x07,
    Key7 = 0x08,
    Key8 = 0x09,
    Key9 = 0x0A,
    Key0 = 0x0B,
    Minus = 0x0C,
    Equals = 0x0D,
    Backspace = 0x0E,
    Tab = 0x0F,
    Q = 0x10,
    W = 0x11,
    E = 0x12,
    R = 0x13,
    T = 0x14,
    Y = 0x15,
    U = 0x16,
    I = 0x17,
    O = 0x18,
    P = 0x19,
    LeftBracket = 0x1A,
    RightBracket = 0x1B,
    Enter = 0x1C,
    LeftControl = 0x1D,
    A = 0x1E,
    S = 0x1F,
    D = 0x20,
    F = 0x21,
    G = 0x22,
    H = 0x23,
    J = 0x24,
    K = 0x25,
    L = 0x26,
    Semicolon = 0x27,
    Quote = 0x28,
    Backtick = 0x29,
    LeftShift = 0x2A,
    Backslash = 0x2B,
    Z = 0x2C,
    X = 0x2D,
    C = 0x2E,
    V = 0x2F,
    B = 0x30,
    N = 0x31,
    M = 0x32,
    Comma = 0x33,
    Dot = 0x34,
    Slash = 0x35,
    RightShift = 0x36,
    KeypadStar = 0x37,
    LeftAlt = 0x38,
    Space = 0x39,
    CapsLock = 0x3A,
    F1 = 0x3B,
    F2 = 0x3C,
    F3 = 0x3D,
    F4 = 0x3E,
    F5 = 0x3F,
    F6 = 0x40,
    F7 = 0x41,
    F8 = 0x42,
    F9 = 0x43,
    F10 = 0x44,
    NumLock = 0x45,
    ScrollLock = 0x46,
    Keypad7 = 0x47,
    Keypad8 = 0x48,
    Keypad9 = 0x49,
    KeypadMinus = 0x4A,
    Keypad4 = 0x4B,
    Keypad5 = 0x4C,
    Keypad6 = 0x4D,
    KeypadPlus = 0x4E,
    Keypad1 = 0x4F,
    Keypad2 = 0x50,
    Keypad3 = 0x51,
    Keypad0 = 0x52,
    KeypadDot = 0x53,
    SysRq = 0x54,
    Fn = 0x55,
    ISO = 0x56,
    F11 = 0x57,
    F12 = 0x58,

    // --- E0 Extended Scan Codes ---
    KeypadEnter = 0xE01C,
    RightControl = 0xE01D,
    KeypadSlash = 0xE035,
    RightAlt = 0xE038,
    Home = 0xE047,
    CursorUp = 0xE048,
    PageUp = 0xE049,
    CursorLeft = 0xE04B,
    CursorRight = 0xE04D,
    End = 0xE04F,
    CursorDown = 0xE050,
    PageDown = 0xE051,
    Insert = 0xE052,
    Delete = 0xE053,
    LeftGUI = 0xE05B,  // Windows Key
    RightGUI = 0xE05C, // Windows Key
    Apps = 0xE05D,     // Menu Key
}

impl KeyCode {
    pub fn from_u16(code: u16) -> Option<Self> {
        let base_code = code & 0xFF7F;

        match base_code {
            0x01 => Some(KeyCode::Escape),
            0x02 => Some(KeyCode::Key1),
            0x03 => Some(KeyCode::Key2),
            0x04 => Some(KeyCode::Key3),
            0x05 => Some(KeyCode::Key4),
            0x06 => Some(KeyCode::Key5),
            0x07 => Some(KeyCode::Key6),
            0x08 => Some(KeyCode::Key7),
            0x09 => Some(KeyCode::Key8),
            0x0A => Some(KeyCode::Key9),
            0x0B => Some(KeyCode::Key0),
            0x0C => Some(KeyCode::Minus),
            0x0D => Some(KeyCode::Equals),
            0x0E => Some(KeyCode::Backspace),
            0x0F => Some(KeyCode::Tab),
            0x10 => Some(KeyCode::Q),
            0x11 => Some(KeyCode::W),
            0x12 => Some(KeyCode::E),
            0x13 => Some(KeyCode::R),
            0x14 => Some(KeyCode::T),
            0x15 => Some(KeyCode::Y),
            0x16 => Some(KeyCode::U),
            0x17 => Some(KeyCode::I),
            0x18 => Some(KeyCode::O),
            0x19 => Some(KeyCode::P),
            0x1A => Some(KeyCode::LeftBracket),
            0x1B => Some(KeyCode::RightBracket),
            0x1C => Some(KeyCode::Enter),
            0x1D => Some(KeyCode::LeftControl),
            0x1E => Some(KeyCode::A),
            0x1F => Some(KeyCode::S),
            0x20 => Some(KeyCode::D),
            0x21 => Some(KeyCode::F),
            0x22 => Some(KeyCode::G),
            0x23 => Some(KeyCode::H),
            0x24 => Some(KeyCode::J),
            0x25 => Some(KeyCode::K),
            0x26 => Some(KeyCode::L),
            0x27 => Some(KeyCode::Semicolon),
            0x28 => Some(KeyCode::Quote),
            0x29 => Some(KeyCode::Backtick),
            0x2A => Some(KeyCode::LeftShift),
            0x2B => Some(KeyCode::Backslash),
            0x2C => Some(KeyCode::Z),
            0x2D => Some(KeyCode::X),
            0x2E => Some(KeyCode::C),
            0x2F => Some(KeyCode::V),
            0x30 => Some(KeyCode::B),
            0x31 => Some(KeyCode::N),
            0x32 => Some(KeyCode::M),
            0x33 => Some(KeyCode::Comma),
            0x34 => Some(KeyCode::Dot),
            0x35 => Some(KeyCode::Slash),
            0x36 => Some(KeyCode::RightShift),
            0x37 => Some(KeyCode::KeypadStar),
            0x38 => Some(KeyCode::LeftAlt),
            0x39 => Some(KeyCode::Space),
            0x3A => Some(KeyCode::CapsLock),
            0x3B => Some(KeyCode::F1),
            0x3C => Some(KeyCode::F2),
            0x3D => Some(KeyCode::F3),
            0x3E => Some(KeyCode::F4),
            0x3F => Some(KeyCode::F5),
            0x40 => Some(KeyCode::F6),
            0x41 => Some(KeyCode::F7),
            0x42 => Some(KeyCode::F8),
            0x43 => Some(KeyCode::F9),
            0x44 => Some(KeyCode::F10),
            0x45 => Some(KeyCode::NumLock),
            0x46 => Some(KeyCode::ScrollLock),
            0x47 => Some(KeyCode::Keypad7),
            0x48 => Some(KeyCode::Keypad8),
            0x49 => Some(KeyCode::Keypad9),
            0x4A => Some(KeyCode::KeypadMinus),
            0x4B => Some(KeyCode::Keypad4),
            0x4C => Some(KeyCode::Keypad5),
            0x4D => Some(KeyCode::Keypad6),
            0x4E => Some(KeyCode::KeypadPlus),
            0x4F => Some(KeyCode::Keypad1),
            0x50 => Some(KeyCode::Keypad2),
            0x51 => Some(KeyCode::Keypad3),
            0x52 => Some(KeyCode::Keypad0),
            0x53 => Some(KeyCode::KeypadDot),
            0x54 => Some(KeyCode::SysRq),
            0x55 => Some(KeyCode::Fn),
            0x56 => Some(KeyCode::ISO),
            0x57 => Some(KeyCode::F11),
            0x58 => Some(KeyCode::F12),

            // Extended Mapping
            0xE01C => Some(KeyCode::KeypadEnter),
            0xE01D => Some(KeyCode::RightControl),
            0xE035 => Some(KeyCode::KeypadSlash),
            0xE038 => Some(KeyCode::RightAlt),
            0xE047 => Some(KeyCode::Home),
            0xE048 => Some(KeyCode::CursorUp),
            0xE049 => Some(KeyCode::PageUp),
            0xE04B => Some(KeyCode::CursorLeft),
            0xE04D => Some(KeyCode::CursorRight),
            0xE04F => Some(KeyCode::End),
            0xE050 => Some(KeyCode::CursorDown),
            0xE051 => Some(KeyCode::PageDown),
            0xE052 => Some(KeyCode::Insert),
            0xE053 => Some(KeyCode::Delete),
            0xE05B => Some(KeyCode::LeftGUI),
            0xE05C => Some(KeyCode::RightGUI),
            0xE05D => Some(KeyCode::Apps),

            _ => None,
        }
    }

    pub fn is_release(code: u16) -> bool {
        (code & 0x80) != 0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KeyChars {
    pub none: char,
    pub shift: char,
    pub alt_gr: char,
}

// Default "null" state for non-character keys
const N: KeyChars = KeyChars { none: '\0', shift: '\0', alt_gr: '\0' };

pub const FINNISH_LOOKUP: [KeyChars; 128] = [
    /* 0x00 */ N,
    /* 0x01 */ N, // Esc
    /* 0x02 */ KeyChars { none: '1',  shift: '!',  alt_gr: '\0' },
    /* 0x03 */ KeyChars { none: '2',  shift: '"',  alt_gr: '@'  },
    /* 0x04 */ KeyChars { none: '3',  shift: '#',  alt_gr: '£'  },
    /* 0x05 */ KeyChars { none: '4',  shift: '¤',  alt_gr: '$'  },
    /* 0x06 */ KeyChars { none: '5',  shift: '%',  alt_gr: '€'  },
    /* 0x07 */ KeyChars { none: '6',  shift: '&',  alt_gr: '{'  },
    /* 0x08 */ KeyChars { none: '7',  shift: '/',  alt_gr: '['  },
    /* 0x09 */ KeyChars { none: '8',  shift: '(',  alt_gr: ']'  },
    /* 0x0A */ KeyChars { none: '9',  shift: ')',  alt_gr: '}'  },
    /* 0x0B */ KeyChars { none: '0',  shift: '=',  alt_gr: '\\' },
    /* 0x0C */ KeyChars { none: '+',  shift: '?',  alt_gr: '`'  },
    /* 0x0D */ KeyChars { none: '´',  shift: '`',  alt_gr: '\0' },
    /* 0x0E */ KeyChars { none: '\x08',  shift: '\x08',  alt_gr: '\x08' }, // Backspace
    /* 0x0F */ KeyChars { none: '\t',  shift: '\t',  alt_gr: '\t' }, // Tab
    /* 0x10 */ KeyChars { none: 'q',  shift: 'Q',  alt_gr: '\0' },
    /* 0x11 */ KeyChars { none: 'w',  shift: 'W',  alt_gr: '\0' },
    /* 0x12 */ KeyChars { none: 'e',  shift: 'E',  alt_gr: '€'  },
    /* 0x13 */ KeyChars { none: 'r',  shift: 'R',  alt_gr: '\0' },
    /* 0x14 */ KeyChars { none: 't',  shift: 'T',  alt_gr: '\0' },
    /* 0x15 */ KeyChars { none: 'y',  shift: 'Y',  alt_gr: '\0' },
    /* 0x16 */ KeyChars { none: 'u',  shift: 'U',  alt_gr: '\0' },
    /* 0x17 */ KeyChars { none: 'i',  shift: 'I',  alt_gr: '\0' },
    /* 0x18 */ KeyChars { none: 'o',  shift: 'O',  alt_gr: '\0' },
    /* 0x19 */ KeyChars { none: 'p',  shift: 'P',  alt_gr: '\0' },
    /* 0x1A */ KeyChars { none: 'å',  shift: 'Å',  alt_gr: '\0' },
    /* 0x1B */ KeyChars { none: '¨',  shift: '^',  alt_gr: '~'  },
    /* 0x1C */ KeyChars { none: '\n', shift: '\n', alt_gr: '\0' }, // Enter
    /* 0x1D */ N, // L-Ctrl
    /* 0x1E */ KeyChars { none: 'a',  shift: 'A',  alt_gr: '\0' },
    /* 0x1F */ KeyChars { none: 's',  shift: 'S',  alt_gr: '\0' },
    /* 0x20 */ KeyChars { none: 'd',  shift: 'D',  alt_gr: '\0' },
    /* 0x21 */ KeyChars { none: 'f',  shift: 'F',  alt_gr: '\0' },
    /* 0x22 */ KeyChars { none: 'g',  shift: 'G',  alt_gr: '\0' },
    /* 0x23 */ KeyChars { none: 'h',  shift: 'H',  alt_gr: '\0' },
    /* 0x24 */ KeyChars { none: 'j',  shift: 'J',  alt_gr: '\0' },
    /* 0x25 */ KeyChars { none: 'k',  shift: 'K',  alt_gr: '\0' },
    /* 0x26 */ KeyChars { none: 'l',  shift: 'L',  alt_gr: '\0' },
    /* 0x27 */ KeyChars { none: 'ö',  shift: 'Ö',  alt_gr: '\0' },
    /* 0x28 */ KeyChars { none: 'ä',  shift: 'Ä',  alt_gr: '\0' },
    /* 0x29 */ KeyChars { none: '§',  shift: '½',  alt_gr: '\0' },
    /* 0x2A */ N, // L-Shift
    /* 0x2B */ KeyChars { none: '\'', shift: '*',  alt_gr: '\0' },
    /* 0x2C */ KeyChars { none: 'z',  shift: 'Z',  alt_gr: '\0' },
    /* 0x2D */ KeyChars { none: 'x',  shift: 'X',  alt_gr: '\0' },
    /* 0x2E */ KeyChars { none: 'c',  shift: 'C',  alt_gr: '\0' },
    /* 0x2F */ KeyChars { none: 'v',  shift: 'V',  alt_gr: '\0' },
    /* 0x30 */ KeyChars { none: 'b',  shift: 'B',  alt_gr: '\0' },
    /* 0x31 */ KeyChars { none: 'n',  shift: 'N',  alt_gr: '\0' },
    /* 0x32 */ KeyChars { none: 'm',  shift: 'M',  alt_gr: '\0' },
    /* 0x33 */ KeyChars { none: ',',  shift: ';',  alt_gr: '\0' },
    /* 0x34 */ KeyChars { none: '.',  shift: ':',  alt_gr: '\0' },
    /* 0x35 */ KeyChars { none: '-',  shift: '_',  alt_gr: '\0' },
    /* 0x36 */ N, // R-Shift
    /* 0x37 */ N, // KP *
    /* 0x38 */ N, // L-Alt
    /* 0x39 */ KeyChars { none: ' ',  shift: ' ',  alt_gr: '\0' }, // Space
    /* 0x3A */ N, // Caps
    // 0x3B to 0x55: F-keys and Keypad
    N, N, N, N, N, N, N, N, N, N, // 0x3B - 0x44 (F1-F10)
    N, N,                         // 0x45 - 0x46 (Num, Scroll Lock)
    N, N, N, N,                   // 0x47 - 0x4A (KP 7, 8, 9, -)
    N, N, N, N,                   // 0x4B - 0x4E (KP 4, 5, 6, +)
    N, N, N, N, N,                // 0x4F - 0x53 (KP 1, 2, 3, 0, .)
    N, N,                         // 0x54 - 0x55 (Alt+SysRq, F11/Reserved)
    /* 0x56 */ KeyChars { none: '<',  shift: '>',  alt_gr: '|'  }, // ISO key
    /* 0x57..0x7F (41 total remaining slots) */
    N, N, N, N, N, N, N, N, N, N,
    N, N, N, N, N, N, N, N, N, N,
    N, N, N, N, N, N, N, N, N, N,
    N, N, N, N, N, N, N, N, N, N, N,
];

fn char_from_lookup(lookup: &[KeyChars; 128], key: KeyCode, shift: bool, alt_gr: bool) -> Option<char> {
    let index = key as u8 as usize;
    let chars = lookup[index];
    let char = match (shift, alt_gr) {
        (false, false) => chars.none,
        (true, false) => chars.shift,
        _ => chars.alt_gr,
    };
    if char == '\0' {None}
    else {Some(char)}
}

pub fn get_finnish_char(key: KeyCode, shift: bool, alt_gr: bool) -> Option<char> {
    char_from_lookup(&FINNISH_LOOKUP, key, shift, alt_gr)
}