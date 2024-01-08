use hashbrown::HashSet;

use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

pub fn get_char_from_event(
    event: &WindowEvent,
    modifiers: &HashSet<VirtualKeyCode>,
) -> Option<char> {
    match event {
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
            ..
        } => {
            let is_pressed = *state == ElementState::Pressed;

            if !is_pressed {
                return None;
            }

            log::warn!("keycode: {:?}", keycode);
            log::warn!("modifiers {:?}", modifiers);

            // let is_shift = modifiers.shift();
            let is_shift = modifiers.contains(&VirtualKeyCode::LShift)
                || modifiers.contains(&VirtualKeyCode::RShift);
            // let is_caps_lock = modifiers.caps_lock();
            let is_caps_lock = false;
            let is_upper = is_shift ^ is_caps_lock;

            let c = match keycode {
                VirtualKeyCode::Key0 => {
                    if is_shift {
                        ')'
                    } else {
                        '0'
                    }
                }
                VirtualKeyCode::Key1 => {
                    if is_shift {
                        '!'
                    } else {
                        '1'
                    }
                }
                VirtualKeyCode::Key2 => {
                    if is_shift {
                        '@'
                    } else {
                        '2'
                    }
                }
                VirtualKeyCode::Key3 => {
                    if is_shift {
                        '#'
                    } else {
                        '3'
                    }
                }
                VirtualKeyCode::Key4 => {
                    if is_shift {
                        '$'
                    } else {
                        '4'
                    }
                }
                VirtualKeyCode::Key5 => {
                    if is_shift {
                        '%'
                    } else {
                        '5'
                    }
                }
                VirtualKeyCode::Key6 => {
                    if is_shift {
                        '^'
                    } else {
                        '6'
                    }
                }
                VirtualKeyCode::Key7 => {
                    if is_shift {
                        '&'
                    } else {
                        '7'
                    }
                }
                VirtualKeyCode::Key8 => {
                    if is_shift {
                        '*'
                    } else {
                        '8'
                    }
                }
                VirtualKeyCode::Key9 => {
                    if is_shift {
                        '('
                    } else {
                        '9'
                    }
                }

                VirtualKeyCode::A => {
                    if is_upper {
                        'A'
                    } else {
                        'a'
                    }
                }
                VirtualKeyCode::B => {
                    if is_upper {
                        'B'
                    } else {
                        'b'
                    }
                }
                VirtualKeyCode::C => {
                    if is_upper {
                        'C'
                    } else {
                        'c'
                    }
                }
                VirtualKeyCode::D => {
                    if is_upper {
                        'D'
                    } else {
                        'd'
                    }
                }
                VirtualKeyCode::E => {
                    if is_upper {
                        'E'
                    } else {
                        'e'
                    }
                }
                VirtualKeyCode::F => {
                    if is_upper {
                        'F'
                    } else {
                        'f'
                    }
                }
                VirtualKeyCode::G => {
                    if is_upper {
                        'G'
                    } else {
                        'g'
                    }
                }
                VirtualKeyCode::H => {
                    if is_upper {
                        'H'
                    } else {
                        'h'
                    }
                }
                VirtualKeyCode::I => {
                    if is_upper {
                        'I'
                    } else {
                        'i'
                    }
                }
                VirtualKeyCode::J => {
                    if is_upper {
                        'J'
                    } else {
                        'j'
                    }
                }
                VirtualKeyCode::K => {
                    if is_upper {
                        'K'
                    } else {
                        'k'
                    }
                }
                VirtualKeyCode::L => {
                    if is_upper {
                        'L'
                    } else {
                        'l'
                    }
                }

                VirtualKeyCode::M => {
                    if is_upper {
                        'M'
                    } else {
                        'm'
                    }
                }
                VirtualKeyCode::N => {
                    if is_upper {
                        'N'
                    } else {
                        'n'
                    }
                }
                VirtualKeyCode::O => {
                    if is_upper {
                        'O'
                    } else {
                        'o'
                    }
                }
                VirtualKeyCode::P => {
                    if is_upper {
                        'P'
                    } else {
                        'p'
                    }
                }
                VirtualKeyCode::Q => {
                    if is_upper {
                        'Q'
                    } else {
                        'q'
                    }
                }
                VirtualKeyCode::R => {
                    if is_upper {
                        'R'
                    } else {
                        'r'
                    }
                }
                VirtualKeyCode::S => {
                    if is_upper {
                        'S'
                    } else {
                        's'
                    }
                }
                VirtualKeyCode::T => {
                    if is_upper {
                        'T'
                    } else {
                        't'
                    }
                }
                VirtualKeyCode::U => {
                    if is_upper {
                        'U'
                    } else {
                        'u'
                    }
                }

                VirtualKeyCode::V => {
                    if is_upper {
                        'V'
                    } else {
                        'v'
                    }
                }
                VirtualKeyCode::W => {
                    if is_upper {
                        'W'
                    } else {
                        'w'
                    }
                }
                VirtualKeyCode::X => {
                    if is_upper {
                        'X'
                    } else {
                        'x'
                    }
                }
                VirtualKeyCode::Y => {
                    if is_upper {
                        'Y'
                    } else {
                        'y'
                    }
                }
                VirtualKeyCode::Z => {
                    if is_upper {
                        'Z'
                    } else {
                        'z'
                    }
                }

                VirtualKeyCode::Apostrophe => {
                    if is_shift {
                        '\"'
                    } else {
                        '\''
                    }
                }
                VirtualKeyCode::Comma => {
                    if is_shift {
                        '<'
                    } else {
                        ','
                    }
                }
                VirtualKeyCode::Period => {
                    if is_shift {
                        '>'
                    } else {
                        '.'
                    }
                }
                VirtualKeyCode::Slash => {
                    if is_shift {
                        '?'
                    } else {
                        '/'
                    }
                }

                VirtualKeyCode::Backslash => {
                    if is_shift {
                        '|'
                    } else {
                        '\\'
                    }
                }
                VirtualKeyCode::Grave => {
                    if is_shift {
                        '~'
                    } else {
                        '`'
                    }
                }
                VirtualKeyCode::Minus => {
                    if is_shift {
                        '_'
                    } else {
                        '-'
                    }
                }
                VirtualKeyCode::Equals => {
                    if is_shift {
                        '+'
                    } else {
                        '='
                    }
                }

                VirtualKeyCode::Semicolon => {
                    if is_shift {
                        ':'
                    } else {
                        ';'
                    }
                }
                VirtualKeyCode::LBracket => {
                    if is_shift {
                        '{'
                    } else {
                        '['
                    }
                }
                VirtualKeyCode::RBracket => {
                    if is_shift {
                        '}'
                    } else {
                        ']'
                    }
                }

                VirtualKeyCode::Space => ' ',
                VirtualKeyCode::Return => '\n',
                _ => return None,
            };
            Some(c)
        }
        _ => None,
    }
}

// use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

// pub fn get_char_from_event(event: &WindowEvent) -> Option<char> {
//     match event {
//         WindowEvent::KeyboardInput {
//             input:
//                 KeyboardInput {
//                     state,
//                     virtual_keycode: Some(keycode),
//                     ..
//                 },
//             ..
//         } => {
//             let is_pressed = *state == ElementState::Pressed;
//             if !is_pressed {
//                 return None;
//             }
//             let c = match keycode {
//                 VirtualKeyCode::Key0 => '0',
//                 VirtualKeyCode::Key1 => '1',
//                 VirtualKeyCode::Key2 => '2',
//                 VirtualKeyCode::Key3 => '3',
//                 VirtualKeyCode::Key4 => '4',
//                 VirtualKeyCode::Key5 => '5',
//                 VirtualKeyCode::Key6 => '6',
//                 VirtualKeyCode::Key7 => '7',
//                 VirtualKeyCode::Key8 => '8',
//                 VirtualKeyCode::Key9 => '9',
//                 VirtualKeyCode::A => 'a',
//                 VirtualKeyCode::B => 'b',
//                 VirtualKeyCode::C => 'c',
//                 VirtualKeyCode::D => 'd',
//                 VirtualKeyCode::E => 'e',
//                 VirtualKeyCode::F => 'f',
//                 VirtualKeyCode::G => 'g',
//                 VirtualKeyCode::H => 'h',
//                 VirtualKeyCode::I => 'i',
//                 VirtualKeyCode::J => 'j',
//                 VirtualKeyCode::K => 'k',
//                 VirtualKeyCode::L => 'l',
//                 VirtualKeyCode::M => 'm',
//                 VirtualKeyCode::N => 'n',
//                 VirtualKeyCode::O => 'o',
//                 VirtualKeyCode::P => 'p',
//                 VirtualKeyCode::Q => 'q',
//                 VirtualKeyCode::R => 'r',
//                 VirtualKeyCode::S => 's',
//                 VirtualKeyCode::T => 't',
//                 VirtualKeyCode::U => 'u',
//                 VirtualKeyCode::V => 'v',
//                 VirtualKeyCode::W => 'w',
//                 VirtualKeyCode::X => 'x',
//                 VirtualKeyCode::Y => 'y',
//                 VirtualKeyCode::Z => 'z',
//                 VirtualKeyCode::Apostrophe => '\'',
//                 VirtualKeyCode::Comma => ',',
//                 VirtualKeyCode::Equals => '=',
//                 VirtualKeyCode::LBracket => '[',
//                 VirtualKeyCode::Minus => '-',
//                 VirtualKeyCode::Period => '.',
//                 VirtualKeyCode::RBracket => ']',
//                 VirtualKeyCode::Semicolon => ';',
//                 VirtualKeyCode::Slash => '/',
//                 VirtualKeyCode::Backslash => '\\',
//                 VirtualKeyCode::Grave => '`',
//                 VirtualKeyCode::Space => ' ',
//                 VirtualKeyCode::Asterisk => '*',
//                 VirtualKeyCode::Plus => '+',
//                 VirtualKeyCode::Return => '\n',
//                 _ => return None,
//             };
//             Some(c)
//         }
//         _ => None,
//     }
// }
