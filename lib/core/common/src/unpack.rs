use leptos::html;
use leptos::html::ElementType;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::Element;

use crate::errors::AppError;

pub fn action_has_error<
    T: Send + Sync + 'static,
    A: Send + Sync + 'static,
>(
    action: Action<A, Result<T, AppError>>
) -> Signal<bool> {
    Signal::derive(move || matches!(*action.value().read(), Some(Err(_))))
}

pub fn handle_initial_load<T: Clone + Send + Sync + 'static>(
    load_result: Result<Vec<T>, AppError>,
    loaded_vec: RwSignal<Vec<T>>,
    load_error: RwSignal<Option<AppError>>,
    list_ref: Option<NodeRef<html::Ul>>,
) {
    match load_result {
        Ok(init_vec) => {
            loaded_vec.set(init_vec);
            if let Some(Some(list_ref)) = list_ref.map(|list_ref| list_ref.get_untracked()) {
                list_ref.set_scroll_top(0);
            }
        },
        Err(ref e) => {
            loaded_vec.write().clear();
            load_error.set(Some(AppError::from(e.clone())))
        },
    };
}

pub fn reset_additional_load<T, R>(
    additional_vec: RwSignal<Vec<T>>,
    additional_load_count: RwSignal<i32>,
    node_ref: Option<NodeRef<R>>,
)
where
    T: Clone + Send + Sync + 'static,
    R: ElementType,
    R::Output: Clone + AsRef<Element> + JsCast + 'static,
{
    additional_vec.write().clear();
    additional_load_count.set(0);
    if let Some(Some(node_ref)) = node_ref.map(|list_ref| list_ref.get_untracked()) {
        if let Some(element) = node_ref.dyn_ref::<Element>() {
            element.set_scroll_top(0);
        }
    }
}

pub fn handle_additional_load<T: Clone + Send + Sync + 'static>(
    mut load_result: Result<Vec<T>, AppError>,
    loaded_vec: RwSignal<Vec<T>>,
    load_error: RwSignal<Option<AppError>>,
) {
    match load_result {
        Ok(ref mut additional_vec) => {
            if !additional_vec.is_empty() {
                loaded_vec.update(|loaded_vec| loaded_vec.append(additional_vec))
            }
        },
        Err(ref e) => load_error.set(Some(AppError::from(e.clone()))),
    }
}

pub fn handle_dialog_action_result<T: Clone + Send + Sync + 'static>(
    action_result: Option<Result<T, AppError>>,
    signal: RwSignal<T>,
    show_dialog: RwSignal<bool>,
) {
    if let Some(Ok(result)) = action_result {
        signal.set(result);
        show_dialog.set(false);
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::AppError;
    use crate::unpack::{handle_additional_load, handle_dialog_action_result, handle_initial_load};
    use leptos::prelude::*;

    #[test]
    fn test_handle_initial_load() {
        let owner = Owner::new();
        owner.set();
        let loaded_vec = RwSignal::new(Vec::new());
        let load_error = RwSignal::new(None);
        
        handle_initial_load(Ok(vec![1, 2, 3]), loaded_vec, load_error, None);
        assert_eq!(loaded_vec.read().as_slice(), &[1, 2, 3]);
        assert_eq!(load_error.read(), None);

        handle_initial_load(Ok(vec![4, 5, 6]), loaded_vec, load_error, None);
        assert_eq!(loaded_vec.read().as_slice(), &[4, 5, 6]);
        assert_eq!(load_error.read(), None);

        let error = AppError::CommunicationError(ServerFnErrorErr::Request(String::from("test")));
        handle_initial_load(Err(error.clone()), loaded_vec, load_error, None);
        assert!(loaded_vec.read().is_empty());
        assert_eq!(load_error.read(), Some(error));
    }

    #[test]
    fn test_handle_additional_load() {
        let owner = Owner::new();
        owner.set();
        let loaded_vec = RwSignal::new(Vec::new());
        let load_error = RwSignal::new(None);

        handle_additional_load(Ok(vec![1, 2, 3]), loaded_vec, load_error);
        assert_eq!(loaded_vec.read().as_slice(), &[1, 2, 3]);
        assert_eq!(load_error.read(), None);

        handle_additional_load(Ok(vec![4, 5, 6]), loaded_vec, load_error);
        assert_eq!(loaded_vec.read().as_slice(), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(load_error.read(), None);

        let error = AppError::CommunicationError(ServerFnErrorErr::Request(String::from("test")));
        handle_additional_load(Err(error.clone()), loaded_vec, load_error);
        assert_eq!(loaded_vec.read().as_slice(), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(load_error.read(), Some(error));
    }

    #[test]
    fn test_handle_dialog_action_result() {
        let owner = Owner::new();
        owner.set();

        let value = RwSignal::new(0);
        let show_dialog = RwSignal::new(true);

        handle_dialog_action_result(None, value, show_dialog);

        assert_eq!(value.get_untracked(), 0);
        assert_eq!(show_dialog.get_untracked(), true);

        handle_dialog_action_result(Some(Err(AppError::NotFound)), value, show_dialog);

        assert_eq!(value.get_untracked(), 0);
        assert_eq!(show_dialog.get_untracked(), true);

        handle_dialog_action_result(Some(Ok(1)), value, show_dialog);

        assert_eq!(value.get_untracked(), 1);
        assert_eq!(show_dialog.get_untracked(), false);
    }
}