use crate::error_template::ErrorTemplate;
use crate::errors::AppError;
use crate::icons::LoadingIcon;
use leptos::prelude::*;
use leptos::server_fn::error::ServerFnErrorErr;

#[component]
pub fn Unpack<
    T, 
    V: IntoView + 'static, 
    F: FnOnce(T) -> V + Send + Sync + 'static
>(
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
    T: Clone + Send + Sync + 'static,
    A: Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(T) -> V + Send + Sync +  'static,
    FB: Fn() -> FV + Send + Sync +  'static,
    FV: IntoView + 'static,
>(
    action: Action<A, Result<T, ServerFnError>>,
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
    }
}

async fn unpack_resource<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, ServerFnError>>,
    children: StoredValue<F>,
) -> impl IntoView {
    match &resource.await {
        Ok(value) => Ok(children.get_value()(value)),
        Err(e) => Err(AppError::from(e)),
    }
}

#[component]
pub fn SuspenseUnpack<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, ServerFnError>>,
    children: F,
) -> impl IntoView {
    let children = StoredValue::new(children);

    view! {
        <Suspense fallback=move || view! { <LoadingIcon/> }>
            <ErrorBoundary fallback=|errors| { view! { <ErrorTemplate errors=errors/> } }>
                {
                    move || Suspend::new(async move { 
                        unpack_resource(resource, children).await
                    })
                }
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn TransitionUnpack<
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    F: Fn(&T) -> V + Clone + Send + Sync + 'static,
>(
    resource: Resource<Result<T, ServerFnError>>,
    children: F,
) -> impl IntoView {
    let children = StoredValue::new(children);

    view! {
        <Transition fallback=move || view! { <LoadingIcon/> }>
            <ErrorBoundary fallback=|errors| { view! { <ErrorTemplate errors=errors/> } }>
                {
                    move || Suspend::new(async move { 
                        unpack_resource(resource, children).await
                    })
                }
            </ErrorBoundary>
        </Transition>
    }
}
