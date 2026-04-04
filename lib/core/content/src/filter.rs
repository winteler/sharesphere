use std::collections::{HashSet};

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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

pub fn on_change_category_input(
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

pub fn on_change_all_categories_input(
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

pub fn on_change_only_categories_input(
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