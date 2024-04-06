use leptos::*;
use crate::icons::LoadingIcon;
use crate::error_template::ErrorTemplate;

#[component]
pub fn Unpack<T, V: IntoView + 'static, F: FnOnce(T) -> V + 'static>(
    what: Option<Result<T, ServerFnError>>,
    children: F,
) -> impl IntoView {
    match what {
        Some(Ok(value)) => Some(Ok(children(value))),
        Some(Err(e)) => Some(Err(ServerFnErrorErr::from(e))),
        None => None,
    }
}

#[component]
pub fn UnpackAction<
    T: Clone + 'static,
    A: 'static,
    V: IntoView + 'static,
    F: Fn(T) -> V + 'static,
    FB: Fn() -> FV + 'static,
    FV: IntoView + 'static,
>(
    action: Action<A, Result<T, ServerFnError>>,
    children: F,
    fallback: FB,
) -> impl IntoView {
    let fallback = store_value(fallback);
    let children = store_value(children);

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
    }
}

#[component]
pub fn UnpackResource<
    T: Clone + 'static,
    A: Clone + 'static,
    V: IntoView + 'static,
    F: Fn(T) -> V + 'static,
>(
    resource: Resource<A, Result<T, ServerFnError>>,
    children: F,
) -> impl IntoView {
    let children = store_value(children);

    view! {
        <Suspense fallback=move || view! { <LoadingIcon/> }>
            <ErrorBoundary fallback=|errors| { view! { <ErrorTemplate errors=errors/> } }>
                {move || match resource.get() {
                    Some(Ok(value)) => Some(Ok(children.with_value(|children| children(value)))),
                    Some(Err(e)) => Some(Err(ServerFnErrorErr::from(e))),
                    None => None,
                }}
            </ErrorBoundary>
        </Suspense>
    }
}