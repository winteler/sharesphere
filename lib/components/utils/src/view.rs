use leptos::prelude::*;

pub trait ToView {
    fn to_view(self) -> impl IntoView + 'static;
}