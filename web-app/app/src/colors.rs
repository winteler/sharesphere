use crate::icons::ArrowUpIcon;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
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
            Color::None => "border border-base-content/20",
            Color::Blue => "bg-blue-500 text-white",
            Color::Purple => "bg-purple-500 text-white",
            Color::Pink => "bg-pink-500 text-white",
            Color::Red => "bg-red-500 text-white",
            Color::Orange => "bg-orange-500 text-black",
            Color::Yellow => "bg-yellow-500 text-black",
            Color::Green => "bg-green-500 text-black",
            Color::Cyan => "bg-cyan-500 text-white",
            Color::Black => "bg-black text-white",
            Color::White => "bg-white text-black",
        }
    }
}

/// Component to select a color
#[component]
pub fn ColorSelect(
    /// Name of the input in the form that contains this component, must correspond to the parameter of the associated server function
    name: &'static str,
    color_input: RwSignal<Color>,
    /// Label of the select
    #[prop(default = "")]
    label: &'static str,
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let color_string = move || color_input.get().to_string();
    let select_class = move || format!("w-4 h-4 {}", color_input.get().to_bg_class());
    let div_class = format!("flex gap-1 {class}");
    view! {
        <div class=div_class>
            <input type="text" name=name value=color_string class="hidden"/>
            <span class="label-text">{label}</span>
            <div class="dropdown dropdown-end">
                <label tabindex="0" class="h-full flex items-center gap-2 p-1 border border-primary hover:bg-base-content/20">
                    <div class=select_class></div>
                    <ArrowUpIcon class="h-2 w-2 rotate-180"/>
                </label>
                <ul tabindex="0" class="menu menu-sm dropdown-content z-[1] p-2 shadow bg-base-200 rounded-sm">
                { move || {
                    Color::iter().into_iter().map(|color: Color| {
                        let div_class = format!("w-4 h-4 rounded-full {}", color.to_bg_class());
                        view! {
                            <li on:click=move |_| color_input.set(color)>
                                <div class=div_class></div>
                            </li>
                        }.into_any()
                    }).collect_view()
                }}
                </ul>
            </div>
        </div>
    }
}