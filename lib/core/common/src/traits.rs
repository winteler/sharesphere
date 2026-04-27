use leptos::prelude::Signal;

pub trait ToLocalizedStr {
    fn to_localized_str(&self) -> Signal<String>;
}