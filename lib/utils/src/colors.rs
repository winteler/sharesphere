use leptos::html;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[cfg(feature = "hydrate")]
use leptos_use::on_click_outside;

use crate::widget::RotatingArrow;

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

/// Component to display a color
#[component]
pub fn ColorIndicator(
    #[prop(into)]
    color: Signal<Color>,
) -> impl IntoView {
    let color_class = move || format!("w-4 h-4 rounded-full {}", color.get().to_bg_class());
    view! {
        <div class="px-2 py-1 h-fit w-fit"><div class=color_class></div></div>
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
    let show_dropdown = RwSignal::new(false);
    let color_string = move || color_input.get().to_string();
    let div_class = format!("h-full flex {class}");
    let dropdown_ref = NodeRef::<html::Div>::new();
    #[cfg(feature = "hydrate")]
    {
        // only enable with "hydrate" to avoid server side "Dropped SendWrapper" error
        let _ = on_click_outside(dropdown_ref, move |_| show_dropdown.set(false));
    }

    let label_view = match label.is_empty() {
        true => None,
        false => Some(view! { <span class="pr-1 label-text">{label}</span> })
    };

    view! {
        <div class=div_class node_ref=dropdown_ref>
            <input type="text" name=name value=color_string class="hidden"/>
            {label_view}
            <div class="h-full w-fit relative">
                <div
                    class="h-full flex items-center 2xl:gap-1 pr-2 border border-primary bg-base-100 hover:bg-base-200"
                    on:click=move |_| show_dropdown.update(|value| *value = !*value)
                >
                    <ColorIndicator color=color_input/>
                    <RotatingArrow point_up=show_dropdown class="h-2 w-2"/>
                </div>
                <Show when=show_dropdown>
                    <div class="absolute z-40 origin-bottom-left">
                        <div class="grid grid-cols-3 gap-1 shadow-sm bg-base-100 rounded-sm mt-1 w-28">
                        { move || {
                            Color::iter().map(|color: Color| {
                                view! {
                                    <div class="w-fit rounded-sm hover:bg-base-200" on:click=move |_| {
                                        color_input.set(color);
                                        show_dropdown.set(false);
                                    }>
                                        <ColorIndicator color/>
                                    </div>
                                }
                            }).collect_view()
                        }}
                        </div>
                    </div>
                </Show>
            </div>
        </div>
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