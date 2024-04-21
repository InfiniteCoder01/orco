//! Unescape a string Based on: https://github.com/BurntSushi/ripgrep/blob/a2e6aec7a4d9382941932245e8854f0ae5703a5e/crates/cli/src/escape.rs#L91
use super::Error;

/// A single state in the state machine used by `unescape`.
#[derive(Clone, Copy, Eq, PartialEq)]
enum State {
    /// The state after seeing a `\`.
    Escape,
    /// The state after seeing a `\x`.
    HexFirst,
    /// The state after seeing a `\x[0-9A-Fa-f]`.
    HexSecond(char),
    /// After seeing \u
    UnicodePrep,
    /// After seeing \u{
    Unicode(u32),
    /// Default state.
    Literal,
}

/// Unescapes a string.
///
/// It supports a limited set of escape sequences:
/// `\n`, `\r`, `\t`, `\\`, `\0`
/// `\xFF` and `\u{FFFFFF}`
pub fn unescape(s: &str, index_offset: usize) -> Result<Vec<u8>, Error> {
    use self::State::*;

    let mut bytes = vec![];
    let mut state = Literal;
    for (index, character) in s.chars().enumerate() {
        let index = index_offset + index;
        match state {
            Literal => match character {
                '\\' => state = Escape,
                character => bytes.extend(character.to_string().as_bytes()),
            },
            Escape => match character {
                'x' => state = HexFirst,
                'n' => {
                    bytes.push(b'\n');
                    state = Literal;
                }
                'r' => {
                    bytes.push(b'\r');
                    state = Literal;
                }
                't' => {
                    bytes.push(b'\t');
                    state = Literal;
                }
                '\\' => {
                    bytes.push(b'\\');
                    state = Literal;
                }
                '0' => {
                    bytes.push(b'\0');
                    state = Literal;
                }
                'u' => state = UnicodePrep,
                character => {
                    return Err(Error::InvalidEscapeCode(
                        index,
                        "escape code, one of 'x', 'n', 'r', 't', '\\', '0', or 'u'",
                        character,
                    ));
                }
            },
            HexFirst => match character {
                '0'..='9' | 'A'..='F' | 'a'..='f' => {
                    state = HexSecond(character);
                }
                character => return Err(Error::InvalidEscapeCode(index, "hex digit", character)),
            },
            HexSecond(first) => match character {
                '0'..='9' | 'A'..='F' | 'a'..='f' => {
                    let ordinal = format!("{}{}", first, character);
                    let byte = u8::from_str_radix(&ordinal, 16).unwrap();
                    bytes.push(byte);
                    state = Literal;
                }
                character => return Err(Error::InvalidEscapeCode(index, "hex digit", character)),
            },
            UnicodePrep => match character {
                '{' => state = Unicode(0),
                character => return Err(Error::InvalidEscapeCode(index, "'{'", character)),
            },
            Unicode(codepoint) => match character {
                '}' => {
                    bytes.extend(
                        char::from_u32(codepoint)
                            .ok_or(Error::InvalidUnicodeCodepoint(index, codepoint))?
                            .to_string()
                            .as_bytes(),
                    );
                    state = Literal;
                }
                '0'..='9' | 'A'..='F' | 'a'..='f' => {
                    state = Unicode(
                        codepoint * 16 + u32::from_str_radix(&character.to_string(), 16).unwrap(),
                    )
                }
                character => return Err(Error::InvalidEscapeCode(index, "hex digit", character)),
            },
        }
    }
    Ok(bytes)
}
