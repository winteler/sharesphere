use leptos::either::Either;
use crate::errors::{AppError, ErrorDisplay};
use crate::icons::LoadingIcon;
use leptos::prelude::*;
use leptos::html;
use leptos::html::ElementType;
use leptos::wasm_bindgen::JsCast;
use web_sys::Element;

pub fn action_has_error<
    T: Send + Sync + 'static,
    A: Send + Sync + 'static,
>(
    action: Action<A, Result<T, AppError>>
) -> Signal<bool> {
    Signal::derive(move || matches!(*action.value().read(), Some(Err(_))))
}

/// Component to render a server action's error
#[component]
pub fn ActionError<
    T: Send + Sync + 'static,
    A: Send + Sync + 'static,
>(
    action: Action<A, Result<T, AppError>>
) -> impl IntoView {
    view! {
        <Show when=action_has_error(action)>
        {
            match &*action.value().read() {
                Some(Err(e)) => view! { <ErrorDisplay error=e.clone()/> }.into_any(),
                _ => ().into_any(),
            }
        }
        </Show>
    }
}

#[component]
pub fn UnpackAction<
    T: Clone + Send + Sync + 'static,
    A: Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(T) -> V + Send + Sync +  'static,
    FB: Fn() -> FV + Send + Sync +  'static,
    FV: IntoView + 'static,
>(
    action: Action<A, Result<T, AppError>>,
    children: F,
    fallback: FB,
) -> impl IntoView {
    let fallback = StoredValue::new(fallback);
    let children = StoredValue::new(children);

    view! {
        <Suspense fallback=move || {
            if action.pending().get() {
                Some(fallback.with_value(|fallback| fallback()))
            } else {
                None
            }
        }>
            {move || match action.value().get() {
                Some(Ok(value)) => Some(Ok(children.with_value(|children| children(value)))),
                Some(Err(e)) => Some(Err(e)),
                None => None,
            }}
        </Suspense>
    }.into_any()
}

async fn unpack_resource<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, AppError>>,
    children: StoredValue<F>,
) -> impl IntoView {
    match &resource.await {
        Ok(value) => Either::Left(children.with_value(|children| children(value))),
        Err(e) => Either::Right(view! { <ErrorDisplay error=e.clone()/> } ),
    }
}

#[component]
pub fn SuspenseUnpack<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, AppError>>,
    #[prop(into, default = Box::new(|| view! { <LoadingIcon/> }.into_any()).into())]
    fallback: ViewFnOnce,
    children: F,
) -> impl IntoView {
    let children = StoredValue::new(children);

    view! {
        <Suspense fallback>
        {
            move || Suspend::new(async move {
                unpack_resource(resource, children).await
            })
        }
        </Suspense>
    }.into_any()
}

#[component]
pub fn TransitionUnpack<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, AppError>>,
    #[prop(into, default = Box::new(|| view! { <LoadingIcon/> }.into_any()).into())]
    fallback: ViewFnOnce,
    children: F,
) -> impl IntoView {
    let children = StoredValue::new(children);

    view! {
        <Transition fallback>
        {
            move || Suspend::new(async move {
                unpack_resource(resource, children).await
            })
        }
        </Transition>
    }.into_any()
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

#[cfg(test)]
mod tests {
    use leptos::prelude::*;
    use crate::errors::AppError;
    use crate::unpack::{handle_additional_load, handle_initial_load};

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
}