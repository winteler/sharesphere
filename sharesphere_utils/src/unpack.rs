use crate::error_template::ErrorTemplate;
use crate::errors::{AppError, ErrorDisplay};
use crate::icons::LoadingIcon;
use leptos::prelude::*;
use leptos::server_fn::error::ServerFnErrorErr;
use leptos::html;

#[component]
pub fn Unpack<
    T,
    V: IntoView + 'static,
    F: FnOnce(T) -> V + Send + Sync + 'static
>(
    what: Option<Result<T, ServerFnError<AppError>>>,
    children: F,
) -> impl IntoView {
    match what {
        Some(Ok(value)) => Some(Ok(children(value))),
        Some(Err(e)) => Some(Err(ServerFnErrorErr::from(e))),
        None => None,
    }
}

pub fn action_has_error<
    T: Send + Sync + 'static,
    A: Send + Sync + 'static,
>(
    action: Action<A, Result<T, ServerFnError<AppError>>>
) -> Signal<bool> {
    Signal::derive(move || matches!(*action.value().read(), Some(Err(_))))
}

/// Component to render a server action's error
#[component]
pub fn ActionError<
    T: Send + Sync + 'static,
    A: Send + Sync + 'static,
>(
    action: Action<A, Result<T, ServerFnError<AppError>>>
) -> impl IntoView {
    view! {
        <Show when=action_has_error(action)>
        {
            match &*action.value().read() {
                Some(Err(e)) => view! { <ErrorDisplay error=e.into()/> }.into_any(),
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
    action: Action<A, Result<T, ServerFnError<AppError>>>,
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
                Some(Err(e)) => Some(Err(ServerFnErrorErr::from(e))),
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
    resource: Resource<Result<T, ServerFnError<AppError>>>,
    children: StoredValue<F>,
) -> impl IntoView {
    match &resource.await {
        Ok(value) => Ok(children.with_value(|children| children(value))),
        Err(e) => Err(AppError::from(e)),
    }
}

#[component]
pub fn SuspenseUnpack<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, ServerFnError<AppError>>>,
    children: F,
) -> impl IntoView {
    let children = StoredValue::new(children);

    view! {
        <Suspense fallback=move || view! { <LoadingIcon/> }.into_any()>
            <ErrorBoundary fallback=|errors| { view! { <ErrorTemplate errors=errors/> }.into_any() }>
                {
                    move || Suspend::new(async move { 
                        unpack_resource(resource, children).await
                    })
                }
            </ErrorBoundary>
        </Suspense>
    }.into_any()
}

#[component]
pub fn TransitionUnpack<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, ServerFnError<AppError>>>,
    children: F,
) -> impl IntoView {
    let children = StoredValue::new(children);

    view! {
        <Transition fallback=move || view! { <LoadingIcon/> }.into_any()>
            <ErrorBoundary fallback=|errors| { view! { <ErrorTemplate errors=errors/> }.into_any() }>
                {
                    move || Suspend::new(async move { 
                        unpack_resource(resource, children).await
                    })
                }
            </ErrorBoundary>
        </Transition>
    }.into_any()
}

pub fn handle_initial_load<T: Clone + Send + Sync + 'static>(
    load_result: Result<Vec<T>, ServerFnError<AppError>>,
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
            load_error.set(Some(AppError::from(e)))
        },
    };
}

pub fn handle_additional_load<T: Clone + Send + Sync + 'static>(
    mut load_result: Result<Vec<T>, ServerFnError<AppError>>,
    loaded_vec: RwSignal<Vec<T>>,
    load_error: RwSignal<Option<AppError>>,
) {
    match load_result {
        Ok(ref mut additional_vec) => loaded_vec.update(|loaded_vec| loaded_vec.append(additional_vec)),
        Err(ref e) => load_error.set(Some(AppError::from(e))),
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::{Owner, Read, RwSignal};
    use server_fn::ServerFnError;
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

        let error = ServerFnError::<AppError>::Request(String::from("test"));
        handle_initial_load(Err(error.clone()), loaded_vec, load_error, None);
        assert!(loaded_vec.read().is_empty());
        assert_eq!(load_error.read(), Some(AppError::from(error)));
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

        let error = ServerFnError::<AppError>::Request(String::from("test"));
        handle_additional_load(Err(error.clone()), loaded_vec, load_error);
        assert_eq!(loaded_vec.read().as_slice(), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(load_error.read(), Some(AppError::from(error)));
    }
}