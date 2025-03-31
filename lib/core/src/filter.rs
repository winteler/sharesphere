use std::collections::{HashMap, HashSet};
use leptos::html;
use leptos::html::Input;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use serde::{Deserialize, Serialize};
use sharesphere_utils::icons::{FiltersIcon};
use sharesphere_utils::unpack::SuspenseUnpack;
use sharesphere_utils::widget::ModalDialog;
use crate::sphere_category::{SphereCategoryBadge};
use crate::state::SphereState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SphereCategoryFilter {
    All,
    CategorySet(CategoryFilterSet),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CategoryFilterSet {
    pub filters: HashSet<i64>,
    pub only_category: bool,
}

impl Default for CategoryFilterSet {
    fn default() -> Self {
        CategoryFilterSet {
            filters: HashSet::new(),
            only_category: true,
        }
    }
}

/// Button to open post filters modal window
#[component]
pub fn PostFiltersButton() -> impl IntoView {
    let show_dialog = RwSignal::new(false);
    let modal_ref = NodeRef::<html::Div>::new();
    let _ = on_click_outside(modal_ref, move |_| show_dialog.set(false));
    let button_class = move || match show_dialog.get() {
        true => "btn max-2xl:btn-sm btn-primary",
        false => "btn max-2xl:btn-sm btn-ghost",
    };
    view! {
        <div class="tooltip" data-tip="Filters">
            <button
                class=button_class
                on:click=move |_| show_dialog.update(|value| *value = !*value)
            >
                <FiltersIcon class="h-4 w-4 2xl:h-7 2xl:w-7"/>
            </button>
        </div>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
            modal_ref
        >
            <div class="bg-base-100 shadow-xl w-fit p-3 rounded-xs flex flex-col gap-3">
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
    let all_input_ref = NodeRef::<Input>::new();
    let without_category_input_ref = NodeRef::<Input>::new();
    let category_input_ref_map = StoredValue::new(HashMap::new());
    view! {
        <div class="flex flex-col gap-1">
            <div class="text-center font-bold text-xl">"Sphere categories"</div>
            <label class="cursor-pointer flex justify-between">
                <span class="label">"All"</span>
                <input
                    type="checkbox"
                    class="toggle toggle-primary"
                    checked=move || sphere_state.sphere_category_filter.read() == SphereCategoryFilter::All
                    node_ref=all_input_ref
                    on:click=move |_| if let Some(input_ref) = all_input_ref.get() {
                        log::info!("All input: {}", input_ref.checked());
                        match input_ref.checked() {
                            true => {
                                // TODO deactivate all other inputs
                                sphere_state.sphere_category_filter.set(SphereCategoryFilter::All);
                            },
                            false => {
                                sphere_state.sphere_category_filter.set(SphereCategoryFilter::CategorySet(CategoryFilterSet::default()));
                            },
                        }
                    }
                />
            </label>
            <div class="w-full border-b border-1"/>
            <SuspenseUnpack resource=sphere_state.sphere_categories_resource let:sphere_category_vec>
            {
                category_input_ref_map.update_value(|mut input_ref_map| input_ref_map.clear());
                sphere_category_vec.iter().map(|sphere_category| {
                    let category_input_ref = NodeRef::<Input>::new();
                    category_input_ref_map.update_value(|mut input_ref_map| {
                        input_ref_map.insert(sphere_category.category_id, category_input_ref);
                    });
                    let category_id = sphere_category.category_id;
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
                                node_ref=category_input_ref
                                on:change=move |_| if let Some(input_ref) = category_input_ref.get() {
                                    log::info!("Category {} input: {}", category_name, input_ref.checked());
                                    match input_ref.checked() {
                                        true => {
                                            sphere_state.sphere_category_filter.update(|filter| {
                                                match filter {
                                                    SphereCategoryFilter::All => *filter = SphereCategoryFilter::CategorySet(CategoryFilterSet {
                                                        filters: HashSet::from([category_id]),
                                                        only_category: false,
                                                    }),
                                                    SphereCategoryFilter::CategorySet(ref mut filter_set) => {
                                                        filter_set.filters.insert(category_id);
                                                    },
                                                }
                                            });
                                        },
                                        false => {
                                            sphere_state.sphere_category_filter.update(|filter| {
                                                if let SphereCategoryFilter::CategorySet(ref mut filter_set) = filter {
                                                    filter_set.filters.remove(&category_id);
                                                }
                                            })
                                        },
                                    }
                                }
                            />
                        </label>
                    }
                }).collect_view()
            }
            </SuspenseUnpack>
            <label class="cursor-pointer flex justify-between">
                <span class="label">"Only categories"</span>
                <input
                    type="checkbox"
                    class="toggle toggle-info"
                    checked=move || sphere_state.sphere_category_filter.read() == SphereCategoryFilter::All
                    node_ref=without_category_input_ref
                    on:click=move |_| if let Some(input_ref) = without_category_input_ref.get() {
                        match input_ref.checked() {
                            true => {
                                sphere_state.sphere_category_filter.update(|filter| match filter {
                                    SphereCategoryFilter::All => *filter = SphereCategoryFilter::CategorySet(CategoryFilterSet::default()),
                                    SphereCategoryFilter::CategorySet(filter_set) => filter_set.only_category = true,
                                });
                            },
                            false => {
                                sphere_state.sphere_category_filter.update(|filter| {
                                    if let SphereCategoryFilter::CategorySet(ref mut filter_set) = filter {
                                        filter_set.only_category = false;
                                    }
                                })
                            },
                        }
                    }
                />
            </label>
        </div>
    }
}