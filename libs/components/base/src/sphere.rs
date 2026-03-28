use leptos::prelude::*;
use leptos::{component, html, server, view, IntoView};
use leptos_fluent::move_tr;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use serde::{Deserialize, Serialize};

use sharesphere_core_common::errors::AppError;
use sharesphere_core_common::icons::SphereIcon;
use sharesphere_core_common::node_utils::has_reached_scroll_load_threshold;
use sharesphere_core_common::routes::get_sphere_path;
use sharesphere_core_common::widget::{LoadIndicators, Badge};
use crate::state::GlobalState;


/// Component to display a sphere's header
#[component]
pub fn SphereHeader(
    sphere_header: SphereHeader
) -> impl IntoView {
    let default_icon_index = sphere_header.sphere_name.as_bytes().first().cloned().unwrap_or_default();
    view! {
        <Badge text=sphere_header.sphere_name>
            <SphereIcon
                icon_url=sphere_header.icon_url
                default_icon_index
                class="content-toolbar-icon-size"
            />
        </Badge>
    }
}

/// Component to display a sphere's header that navigates to it upon clicking
#[component]
pub fn SphereHeaderLink(
    sphere_header: SphereHeader
) -> impl IntoView {
    // use navigate and prevent default to handle case where sphere header is in another <a>
    let navigate = use_navigate();
    let sphere_name = StoredValue::new(sphere_header.sphere_name.clone());
    let sphere_path = get_sphere_path(&sphere_header.sphere_name);
    let aria_label = move_tr!("navigate-sphere", {"sphere_name" => sphere_name.get_value()});
    view! {
        <button
            class="button-rounded-neutral px-2 py-1 flex items-center"
            on:click=move |ev| {
                ev.prevent_default();
                navigate(sphere_path.as_str(), NavigateOptions::default());
            }
            aria-label=aria_label
        >
            <SphereHeader sphere_header/>
        </button>
    }
}

/// Component to display a collapsable list of sphere links
#[component]
pub fn SphereLinkItems(
    sphere_header_vec: Vec<SphereHeader>,
    #[prop(default = true)]
    is_dropdown_style: bool,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let item_class = match is_dropdown_style {
        true => "px-1 2xl:px-2 py-1 my-1 rounded-sm hover:bg-base-content/20 overflow-hidden",
        false => "px-1 2xl:px-2 py-1 my-1 rounded-sm hover:bg-base-200 overflow-hidden"
    };
    view! {
        <For
            each= move || sphere_header_vec.clone().into_iter()
            key=|sphere_header| sphere_header.sphere_name.clone()
            children=move |sphere_header| {
                let sphere_path = get_sphere_path(&sphere_header.sphere_name);
                view! {
                    <li>
                        <a
                            href=sphere_path
                            on:click=move |_| state.show_left_sidebar.set(false)
                        >
                            <div class=item_class>
                                <SphereHeader sphere_header=sphere_header/>
                            </div>
                        </a>
                    </li>
                }
            }
        />
    }
}

/// Component to display a list of sphere links
#[component]
pub fn SphereLinkList(
    sphere_header_vec: Vec<SphereHeader>
) -> impl IntoView {
    if sphere_header_vec.is_empty() {
        return ().into_any()
    }
    view! {
        <ul class="flex flex-col 2xl:p-1">
            <SphereLinkItems sphere_header_vec/>
        </ul>
    }.into_any()
}

/// Component to display a collapsable list of sphere links
#[component]
pub fn InfiniteSphereLinkList(
    /// signal containing the sphere headers to display
    #[prop(into)]
    sphere_header_vec: Signal<Vec<SphereHeader>>,
    #[prop(into)]
    is_loading: Signal<bool>,
    /// signal containing an eventual loading error in order to display it
    #[prop(into)]
    load_error: Signal<Option<AppError>>,
    /// signal to request loading additional sphere headers
    additional_load_count: RwSignal<i32>,
    /// boolean to style the links for a dropdown
    #[prop(optional)]
    is_dropdown_style: bool,
    /// reference to the container of the sphere headers in order to reset scroll position when context changes
    list_ref: NodeRef<html::Ul>,
) -> impl IntoView {
    let (list_class, load_div_class) = match is_dropdown_style {
        true => (
            "flex flex-col overflow-y-auto max-h-124 w-full p-1",
            "w-full min-h-0",
        ),
        false => (
            "flex flex-col overflow-y-auto max-h-full w-full p-1",
            "w-full min-h-9 lg:min-h-17",
        ),
    };
    view! {
        <Show when=move || !sphere_header_vec.read().is_empty()>
            <ul class=list_class
                on:scroll=move |_| if has_reached_scroll_load_threshold(list_ref) && !is_loading.get_untracked() {
                    additional_load_count.update(|value| *value += 1);
                }
                node_ref=list_ref
            >
                <SphereLinkItems sphere_header_vec=sphere_header_vec.get() is_dropdown_style/>
                <li><LoadIndicators load_error is_loading load_div_class/></li>
            </ul>
        </Show>
    }.into_any()
}