use std::collections::HashSet;
use leptos::html;
use leptos::html::Input;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use serde::{Deserialize, Serialize};
use sharesphere_utils::icons::FiltersIcon;
use sharesphere_utils::unpack::SuspenseUnpack;
use sharesphere_utils::widget::ModalDialog;
use crate::sphere_category::{SphereCategoryBadge, SphereCategoryHeader};
use crate::state::SphereState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SphereCategoryFilter {
    All,
    CategorySet((Vec<i64>, bool)),
}

/// Button to open post filters modal window
#[component]
pub fn PostFiltersButton() -> impl IntoView {
    let show_dialog = RwSignal::new(false);
    let modal_ref = NodeRef::<html::Div>::new();
    let _ = on_click_outside(modal_ref, move |_| show_dialog.set(false));
    let button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <button
            class=button_class
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            <FiltersIcon class="h-4 w-4 2xl:h-7 2xl:w-7"/>
        </button>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
            modal_ref
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Post filters"</div>
                <SphereCategoryFilter/>
                <button
                    type="button"
                    class="btn btn-error"
                    on:click=move |_| show_dialog.set(false)
                >
                    "Close"
                </button>
            </div>
        </ModalDialog>
    }
}

/// Button to open post filters modal window
#[component]
pub fn SphereCategoryFilter() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    //let sphere_category_set = StoredValue::new(HashSet::new());
    let all_input_ref = NodeRef::<Input>::new();
    view! {
        <div class="flex flex-col gap-1">
            <div class="text-center font-bold text-xl">"Sphere categories"</div>
            <label class="cursor-pointer flex justify-between">
                <span class="label">"All"</span>
                <input
                    type="checkbox"
                    class="toggle toggle-primary"
                    checked=move || sphere_state.sphere_category_filter.read() == SphereCategoryFilter::All
                    on:click=move |_| if let Some(input_ref) = all_input_ref.get() {
                        log::info!("All input: {}", input_ref.checked())
                    }
                    node_ref=all_input_ref
                />
            </label>
            <SuspenseUnpack resource=sphere_state.sphere_categories_resource let:sphere_category_vec>
            {
                let input_ref = NodeRef::<Input>::new();
                sphere_category_vec.iter().map(|sphere_category| {
                    let category_name = sphere_category.category_name.clone();
                    view! {
                        <label class="cursor-pointer flex justify-between">
                            <span class="label">
                                <SphereCategoryBadge category_header=sphere_category.into()/>
                            </span>
                            <input
                                type="checkbox"
                                class="toggle toggle-secondary"
                                checked=move || sphere_state.sphere_category_filter.read() == SphereCategoryFilter::All
                                on:change=move |_| if let Some(input_ref) = input_ref.get() {
                                    log::info!("Category {} input: {}", category_name, input_ref.checked())
                                }
                                node_ref=input_ref
                            />
                        </label>
                    }
                }).collect_view()
            }
            </SuspenseUnpack>
        </div>
    }
}