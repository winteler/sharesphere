#[cfg(feature = "ssr")]
use http::StatusCode;
use leptos::prelude::*;

use crate::errors::{AppError, ErrorDisplay};

// A basic function to display errors served by the error boundaries. Feel free to do more complicated things
// here than just displaying them
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<ArcRwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => ArcRwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // Get Errors from Signal
    let errors = errors.get_untracked();

    log::debug!("Error template: got errors: {errors:?}");
    // Downcast lets us take a type that implements `std::error::Error`
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();
    log::debug!("Error template: got errors after downcast: {errors:#?}");

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            let status_code = match errors.first() {
                Some(error) => error.status_code(),
                None => StatusCode::INTERNAL_SERVER_ERROR,
            };
            response.set_status(status_code);
        }
    }

    view! {
        <div class="w-full h-full flex flex-col items-center justify-center">
            <For
                each= move || {errors.clone().into_iter().enumerate()}
                key=|(index, _error)| *index
                children=move |(_, error)| {
                    view! { <ErrorDisplay error/> }
                }
            />
        </div>
    }
}
