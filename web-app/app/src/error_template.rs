#[cfg(feature = "ssr")]
use http::StatusCode;
use leptos::*;

use crate::errors::AppError;

// A basic function to display errors served by the error boundaries. Feel free to do more complicated things
// here than just displaying them
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => create_rw_signal(e),
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
        <h1>{if errors.len() > 1 {"Errors"} else {"Error"}}</h1>
        <For
            // a function that returns the items we're iterating over; a signal is fine
            each= move || {errors.clone().into_iter().enumerate()}
            // a unique key for each item as a reference
            key=|(index, _error)| *index
            // renders each item to a view
            children=move |error| {
                let error = error.1;
                let error_string = error.to_string();
                let status_code =  error.status_code();
                log::error!("Caught error in ErrorTemplate, status_code: {status_code}, error message: {error_string}");
                view! {
                    <h2>{status_code.to_string()}</h2>
                }
            }
        />
    }
}
