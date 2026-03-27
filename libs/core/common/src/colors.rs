use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[repr(i16)]
#[derive(Clone, Copy, Debug, Display, EnumIter, Eq, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum Color {
    None = -1,
    Blue = 0,
    Purple = 1,
    Pink = 2,
    Red = 3,
    Orange = 4,
    Yellow = 5,
    Green = 6,
    Cyan = 7,
    Black = 8,
    White = 9,
}

impl From<i16> for Color {
    fn from(category_color_val: i16) -> Self {
        match category_color_val {
            x if x == Color::Blue as i16 => Color::Blue,
            x if x == Color::Purple as i16 => Color::Purple,
            x if x == Color::Pink as i16 => Color::Pink,
            x if x == Color::Red as i16 => Color::Red,
            x if x == Color::Orange as i16 => Color::Orange,
            x if x == Color::Yellow as i16 => Color::Yellow,
            x if x == Color::Green as i16 => Color::Green,
            x if x == Color::Cyan as i16 => Color::Cyan,
            x if x == Color::Black as i16 => Color::Black,
            x if x == Color::White as i16 => Color::White,
            _ => Color::None,
        }
    }
}

impl Color {
    pub fn to_bg_class(&self) -> &'static str {
        match self {
            Color::None => "border border-base-content/20 font-semibold",
            Color::Blue => "bg-blue-600 text-white font-semibold",
            Color::Purple => "bg-purple-600 text-white font-semibold",
            Color::Pink => "bg-pink-600 text-white font-semibold",
            Color::Red => "bg-red-600 text-black font-semibold",
            Color::Orange => "bg-orange-600 text-black font-semibold",
            Color::Yellow => "bg-yellow-600 text-black font-semibold",
            Color::Green => "bg-green-600 text-black font-semibold",
            Color::Cyan => "bg-cyan-600 text-white font-semibold",
            Color::Black => "bg-black text-white font-semibold",
            Color::White => "bg-white text-black font-semibold",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::colors::Color;
    use strum::IntoEnumIterator;

    #[test]
    fn test_color_from_i16() {
        for color in Color::iter() {
            assert_eq!(Color::from(color as i16), color);
        }
        assert_eq!(Color::from(-2), Color::None);
        assert_eq!(Color::from(100), Color::None);
    }
}