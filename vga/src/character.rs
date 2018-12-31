#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Character {
    character: u8,
    attribute: u8,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    Gray = 0x7,
    DarkGray = 0x8,
    BrightBlue = 0x9,
    BrightGreen = 0xA,
    BrightCyan = 0xB,
    BrightRed = 0xC,
    BrightMagenta = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

impl Character {
    pub fn new(character: u8, foreground: Color, background: Color) -> Character {
        let attribute = ((background as u8) << 4) + (foreground as u8);

        Character {
            character,
            attribute,
        }
    }

    pub fn as_ushort(&self) -> (u16) {
        (self.character as u16) | ((self.attribute as u16) << 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let character = Character::new(b'a', Color::Blue, Color::BrightMagenta);

        assert_eq!(character.character, b'a');
        assert_eq!(character.attribute, 0xD1);
        assert_eq!(character.as_ushort(), b'a' as u16 | 0xD100);

        let character = Character::new(b'b', Color::Yellow, Color::Red);

        assert_eq!(character.character, b'b');
        assert_eq!(character.attribute, 0x4E);

        let character = Character::new(b'c', Color::DarkGray, Color::White);

        assert_eq!(character.character, b'c');
        assert_eq!(character.attribute, 0xF8);
    }

}
