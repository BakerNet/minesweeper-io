use http::status::StatusCode;
use leptos::prelude::*;
use thiserror::Error;

#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
    #[allow(dead_code)]
    #[error("Not Logged In")]
    NotLoggedIn,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::NotLoggedIn => StatusCode::FORBIDDEN,
            // _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// A basic function to display errors served by the error boundaries.
// Feel free to do more complicated things here than just displaying the error.
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // Get Errors from Signal
    let errors = errors.get_untracked();

    // Downcast lets us take a type that implements `std::error::Error`
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();
    println!("Errors: {errors:#?}");

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    #[cfg(feature = "ssr")]
    {
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }

    view! {
        <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">

            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">
                {if errors.len() > 1 { "Errors" } else { "Error" }}
            </h1>
            <For
                // a function that returns the items we're iterating over; a signal is fine
                each=move || { errors.clone().into_iter().enumerate() }
                // a unique key for each item as a reference
                key=|(index, _error)| *index
                // renders each item to a view
                children=move |error| {
                    let error_string = error.1.to_string();
                    let error_code = error.1.status_code();
                    view! {
                        <h2>{error_code.to_string()}</h2>
                        <p>"Error: " {error_string}</p>
                    }
                }
            />
        </div>
    }
}
