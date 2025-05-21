use std::collections::{HashMap, HashSet};
use leptos::html::{Div, Input};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "hydrate")]
use leptos_use::on_click_outside;

use sharesphere_utils::icons::{FiltersIcon};
use sharesphere_utils::unpack::SuspenseUnpack;
use sharesphere_utils::widget::{Dropdown};
use crate::sphere_category::{SphereCategoryBadge};
use crate::state::SphereState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SphereCategoryFilter {
    All,
    CategorySet(CategorySetFilter),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CategorySetFilter {
    #[serde(default)]
    pub filters: HashSet<i64>,
    pub only_category: bool,
}

impl Default for CategorySetFilter {
    fn default() -> Self {
        CategorySetFilter {
            filters: HashSet::new(),
            only_category: true,
        }
    }
}

impl CategorySetFilter {
    pub fn new(category_id: i64) -> Self
    {
        CategorySetFilter {
            filters: std::iter::once(category_id).collect(),
            only_category: true,
        }
    }
}

/// Button to open post filters modal window
#[component]
pub fn PostFiltersButton() -> impl IntoView {
    let show_dropdown = RwSignal::new(false);
    let dropdown_ref = NodeRef::<Div>::new();
    #[cfg(feature = "hydrate")]
    {
        // only enable with "hydrate" to avoid server side "Dropped SendWrapper" error
        let _ = on_click_outside(dropdown_ref, move |_| show_dropdown.set(false));
    }
    let button_class = move || match show_dropdown.get() {
        true => "button-primary",
        false => "button-ghost",
    };
    view! {
        <div class="h-full relative" node_ref=dropdown_ref>
            <div class="tooltip" data-tip="Filters">
                <button
                    class=button_class
                    on:click=move |_| show_dropdown.update(|value| *value = !*value)
                >
                    <FiltersIcon/>
                </button>
            </div>
            <Dropdown show_dropdown>
                <div class="bg-base-200 shadow-xl my-1 p-3 rounded-xs flex flex-col gap-3">
                    <div class="text-center font-bold text-2xl">"Post filters"</div>
                    <SphereCategoryFilter/>
                </div>
            </Dropdown>
        </div>
    }
}

/// Button to open post filters modal window
#[component]
pub fn SphereCategoryFilter() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let all_input_ref = NodeRef::<Input>::new();
    let only_category_input_ref = NodeRef::<Input>::new();
    let category_input_ref_map = StoredValue::new(HashMap::<i64, NodeRef<Input>>::new());
    view! {
        <div class="flex flex-col gap-1">
            <div class="text-center font-bold text-xl whitespace-nowrap">"Sphere categories"</div>
            <label class="cursor-pointer flex justify-between">
                <span class="label">"All"</span>
                <input
                    type="checkbox"
                    class="toggle toggle-primary"
                    checked=true
                    node_ref=all_input_ref
                    on:change=move |_| on_change_all_category_input(sphere_state.sphere_category_filter, all_input_ref, only_category_input_ref, category_input_ref_map)
                />
            </label>
            <div class="w-full border-b border-0.5 border-base-content/20"/>
            <SuspenseUnpack resource=sphere_state.sphere_categories_resource let:sphere_category_vec>
            {
                category_input_ref_map.update_value(|mut input_ref_map| input_ref_map.clear());
                sphere_category_vec.iter().map(|sphere_category| {
                    let category_input_ref = NodeRef::<Input>::new();
                    category_input_ref_map.update_value(|mut input_ref_map| {
                        input_ref_map.insert(sphere_category.category_id, category_input_ref);
                    });
                    let category_id = sphere_category.category_id;
                    view! {
                        <label class="cursor-pointer flex justify-between">
                            <span class="label">
                                <SphereCategoryBadge category_header=sphere_category.into()/>
                            </span>
                            <input
                                type="checkbox"
                                class="toggle toggle-secondary"
                                checked=false
                                node_ref=category_input_ref
                                on:change=move |_| on_change_category_input(sphere_state.sphere_category_filter, all_input_ref, only_category_input_ref, category_input_ref, category_id)
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
                    checked=false
                    node_ref=only_category_input_ref
                    disabled=move || sphere_state.sphere_category_filter.read() == SphereCategoryFilter::All
                    on:change=move |_| on_change_only_category_input(sphere_state.sphere_category_filter, only_category_input_ref)
                />
            </label>
        </div>
    }
}

fn on_change_all_category_input(
    sphere_category_filter: RwSignal<SphereCategoryFilter>,
    all_input_ref: NodeRef<Input>,
    only_category_input_ref: NodeRef<Input>,
    category_input_ref_map: StoredValue<HashMap<i64, NodeRef<Input>>>,
) {
    if let Some(input_ref) = all_input_ref.get() {
        match input_ref.checked() {
            true => {
                category_input_ref_map.with_value(|input_ref_map| {
                    for input_ref in input_ref_map.values() {
                        if let Some(input_ref) = input_ref.get() {
                            input_ref.set_checked(false);
                        }
                    }
                });
                if let Some(input_ref) = only_category_input_ref.get() {
                    input_ref.set_checked(false);
                }
                sphere_category_filter.set(SphereCategoryFilter::All);
            },
            false => {
                sphere_category_filter.set(SphereCategoryFilter::CategorySet(CategorySetFilter {
                    filters: Default::default(),
                    only_category: false,
                }));
            },
        }
    }
}

fn on_change_category_input(
    sphere_category_filter: RwSignal<SphereCategoryFilter>,
    all_input_ref: NodeRef<Input>,
    only_category_input_ref: NodeRef<Input>,
    category_input_ref: NodeRef<Input>,
    category_id: i64,
) {
    if let Some(input_ref) = category_input_ref.get() {
        match input_ref.checked() {
            true => {
                sphere_category_filter.update(|filter| {
                    match filter {
                        SphereCategoryFilter::All => {
                            if let Some(all_input_ref) = all_input_ref.get() {
                                all_input_ref.set_checked(false);
                            }
                            if let Some(input_ref) = only_category_input_ref.get() {
                                input_ref.set_checked(true);
                            }
                            *filter = SphereCategoryFilter::CategorySet(CategorySetFilter::new(category_id));
                        },
                        SphereCategoryFilter::CategorySet(ref mut filter_set) => {
                            filter_set.filters.insert(category_id);
                        },
                    }
                });
            },
            false => {
                sphere_category_filter.update(|filter| {
                    if let SphereCategoryFilter::CategorySet(ref mut filter_set) = filter {
                        filter_set.filters.remove(&category_id);
                    }
                })
            },
        }
    }
}

fn on_change_only_category_input(
    sphere_category_filter: RwSignal<SphereCategoryFilter>,
    only_category_input_ref: NodeRef<Input>,
) {
    if let Some(input_ref) = only_category_input_ref.get() {
        match input_ref.checked() {
            true => {
                sphere_category_filter.update(|filter| match filter {
                    SphereCategoryFilter::All => *filter = SphereCategoryFilter::CategorySet(CategorySetFilter::default()),
                    SphereCategoryFilter::CategorySet(filter_set) => filter_set.only_category = true,
                });
            },
            false => {
                sphere_category_filter.update(|filter| {
                    if let SphereCategoryFilter::CategorySet(ref mut filter_set) = filter {
                        filter_set.only_category = false;
                    }
                })
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::CategorySetFilter;

    #[test]
    fn test_category_set_filter_default() {
        let default_category_filter = CategorySetFilter::default();
        assert!(default_category_filter.filters.is_empty());
        assert!(default_category_filter.only_category);
    }

    #[test]
    fn test_category_set_filter_new() {
        let default_category_filter = CategorySetFilter::new(7);
        assert_eq!(default_category_filter.filters.len(), 1);
        assert_eq!(default_category_filter.filters.iter().next(), Some(&7));
        assert!(default_category_filter.only_category);
    }
}