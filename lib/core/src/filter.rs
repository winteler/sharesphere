use std::collections::{HashSet};
use leptos::html::{Div, Input};
use leptos::prelude::*;
use leptos_fluent::move_tr;
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
            <div class="tooltip" data-tip=move_tr!("filters")>
                <button
                    class=button_class
                    on:click=move |_| show_dropdown.update(|value| *value = !*value)
                >
                    <FiltersIcon/>
                </button>
            </div>
            <Dropdown show_dropdown>
                <div class="bg-base-200 shadow-xl my-1 p-3 rounded-xs flex flex-col gap-3">
                    <div class="text-center font-bold text-2xl">{move_tr!("category-filters")}</div>
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
    view! {
        <div class="flex flex-col gap-1">
            <div class="text-center font-bold text-xl whitespace-nowrap">{move_tr!("sphere-categories")}</div>
            <SuspenseUnpack resource=sphere_state.sphere_categories_resource let:sphere_category_vec>
            {
                sphere_category_vec.iter().map(|sphere_category| {
                    let category_id = sphere_category.category_id;
                    view! {
                        <label class="cursor-pointer flex justify-between">
                            <span class="label">
                                <SphereCategoryBadge category_header=sphere_category/>
                            </span>
                            <SphereCategoryToggle category_id/>
                        </label>
                    }
                }).collect_view()
            }
            </SuspenseUnpack>
            <div class="w-full border-b border-0.5 border-base-content/20"/>
            <AllCategoriesToggle/>
            <OnlyCategoriesToggle/>
        </div>
    }
}

#[component]
pub fn SphereCategoryToggle(category_id: i64) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let input_ref = NodeRef::<Input>::new();
    let is_filter_active = move || {
        let is_active = match &*sphere_state.sphere_category_filter.read() {
            SphereCategoryFilter::All => false,
            SphereCategoryFilter::CategorySet(category_set) => category_set.filters.contains(&category_id),
        };
        set_checkbox(is_active, input_ref);
        is_active
    };

    view! {
        <input
            type="checkbox"
            class="toggle toggle-secondary"
            checked=is_filter_active
            on:change=move |_| on_change_category_input(sphere_state.sphere_category_filter, category_id)
            node_ref=input_ref
        />
    }
}

#[component]
pub fn AllCategoriesToggle() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let input_ref = NodeRef::<Input>::new();
    let is_filter_all_categories = move || {
        let is_active = sphere_state.sphere_category_filter.read() == SphereCategoryFilter::All;
        set_checkbox(is_active, input_ref);
        is_active
    };
    view! {
        <label class="cursor-pointer flex justify-between">
            <span class="label">{move_tr!("all")}</span>
            <input
                type="checkbox"
                class="toggle toggle-primary"
                checked=is_filter_all_categories
                on:change=move |_| on_change_all_categories_input(sphere_state.sphere_category_filter)
                node_ref=input_ref
            />
        </label>
    }
}

#[component]
pub fn OnlyCategoriesToggle() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let input_ref = NodeRef::<Input>::new();
    let is_filter_only_categories = move || {
        let is_active = match &*sphere_state.sphere_category_filter.read() {
            SphereCategoryFilter::All => false,
            SphereCategoryFilter::CategorySet(category_set) => category_set.only_category,
        };
        set_checkbox(is_active, input_ref);
        is_active
    };
    view! {
        <label class="cursor-pointer flex justify-between">
            <span class="label">{move_tr!("only-categories")}</span>
            <input
                type="checkbox"
                class="toggle toggle-primary"
                checked=is_filter_only_categories
                on:change=move |_| on_change_only_categories_input(sphere_state.sphere_category_filter)
                node_ref=input_ref
            />
        </label>
    }
}

fn set_checkbox(is_checked: bool, input_ref: NodeRef<Input>) {
    if let Some(input_ref) = input_ref.get() {
        input_ref.set_checked(is_checked);
    }
}

fn on_change_category_input(
    sphere_category_filter: RwSignal<SphereCategoryFilter>,
    category_id: i64,
) {
    let mut category_filter = sphere_category_filter.write();
    match &mut *category_filter {
        SphereCategoryFilter::All => {
            let new_category_filter = SphereCategoryFilter::CategorySet(
                CategorySetFilter::new(category_id)
            );
            *category_filter = new_category_filter;
        },
        SphereCategoryFilter::CategorySet(category_set) => {
            if !category_set.filters.remove(&category_id) {
                category_set.filters.insert(category_id);
            }
        },
    };

}

fn on_change_all_categories_input(
    sphere_category_filter: RwSignal<SphereCategoryFilter>,
) {
    let mut category_filter = sphere_category_filter.write();
    match &mut *category_filter {
        SphereCategoryFilter::All => {
            *category_filter = SphereCategoryFilter::CategorySet(CategorySetFilter::default());
        },
        SphereCategoryFilter::CategorySet(_) => {
            *category_filter = SphereCategoryFilter::All;
        },
    };
}

fn on_change_only_categories_input(
    sphere_category_filter: RwSignal<SphereCategoryFilter>,
) {
    let mut current_filter = sphere_category_filter.write();
    match &mut *current_filter {
        SphereCategoryFilter::All => {
            *current_filter = SphereCategoryFilter::CategorySet(CategorySetFilter::default());
        },
        SphereCategoryFilter::CategorySet(category_set) => {
            category_set.only_category = !category_set.only_category;
        },
    };
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;
    use crate::filter::{on_change_all_categories_input, on_change_category_input, on_change_only_categories_input, CategorySetFilter, SphereCategoryFilter};

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

    #[test]
    fn test_on_change_category_input() {
        let owner = Owner::new();
        owner.set();
        let sphere_category_filter = RwSignal::new(SphereCategoryFilter::All);

        on_change_category_input(sphere_category_filter, 0);
        let mut expected_category_set = CategorySetFilter::new(0);
        assert_eq!(
            sphere_category_filter.get(),
            SphereCategoryFilter::CategorySet(expected_category_set.clone()),
        );

        on_change_category_input(sphere_category_filter, 1);
        expected_category_set.filters.insert(1);

        assert_eq!(
            sphere_category_filter.get(),
            SphereCategoryFilter::CategorySet(expected_category_set.clone()),
        );

        on_change_category_input(sphere_category_filter, 0);
        expected_category_set.filters.remove(&0);

        assert_eq!(
            sphere_category_filter.get(),
            SphereCategoryFilter::CategorySet(expected_category_set),
        );
    }

    #[test]
    fn test_on_change_all_categories_input() {
        let owner = Owner::new();
        owner.set();
        let sphere_category_filter = RwSignal::new(SphereCategoryFilter::All);

        on_change_all_categories_input(sphere_category_filter);
        let expected_category_set = CategorySetFilter::default();
        assert_eq!(
            sphere_category_filter.get(),
            SphereCategoryFilter::CategorySet(expected_category_set),
        );

        on_change_all_categories_input(sphere_category_filter);

        assert_eq!(
            sphere_category_filter.get(),
            SphereCategoryFilter::All,
        );
    }

    #[test]
    fn test_on_change_only_categories_input() {
        let owner = Owner::new();
        owner.set();
        let sphere_category_filter = RwSignal::new(SphereCategoryFilter::All);

        on_change_only_categories_input(sphere_category_filter);
        let mut expected_category_set = CategorySetFilter::default();
        assert_eq!(
            sphere_category_filter.get(),
            SphereCategoryFilter::CategorySet(expected_category_set.clone()),
        );

        if let SphereCategoryFilter::CategorySet(category_set) = &mut *sphere_category_filter.write() {
            category_set.filters.insert(0);
        }

        on_change_only_categories_input(sphere_category_filter);
        expected_category_set.only_category = false;
        expected_category_set.filters.insert(0);

        assert_eq!(
            sphere_category_filter.get(),
            SphereCategoryFilter::CategorySet(expected_category_set),
        );
    }
}