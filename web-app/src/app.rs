use std::ops::Div;
use crate::error_template::{AppError, ErrorTemplate};
use crate::navigation_bar::*;
use crate::drawer::*;
//use crate::footer::*;
use crate::auth::*;
use leptos::*;
use leptos::ev::Event;
use leptos_meta::*;
use leptos_router::*;

#[derive(Copy, Clone, Debug)]
struct GlobalState {
    temp: RwSignal<bool>,
}

impl GlobalState {
    pub fn new(cx: Scope) -> Self {
        Self {
            temp: create_rw_signal(cx, false)
        }
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    // Provide global context for app
    provide_context(cx, GlobalState::new(cx));

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/start-axum.css"/>

        // sets the document title
        <Title text="Welcome to [[ProjectName]]"/>

        // content for this welcome page
        <Router fallback=|cx| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { cx,
                <ErrorTemplate outside_errors/>
            }
            .into_view(cx)
        }>
            <div class="h-screen flex flex-col">
                <NavigationBar/>
                <main class="h-full drawer lg:drawer-open">
                    <input id="my-drawer" type="checkbox" class="drawer-toggle"/>
                    <div class="drawer-content container mx-auto h-full">
                        <Routes>
                            <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                            <Route path="/auth-callback" view=|cx| view! { cx, <AuthCallback/> }/>
                        </Routes>
                    </div>
                    <div class="drawer-side">
                        <Drawer/>
                    </div>
                </main>
                //<Footer/>
            </div>
        </Router>
    }
}




/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);

    view! { cx,
        <div class="h-full my-0 mx-auto max-w-3xl text-center">
            <h2 class="p-6 text-4xl">"Welcome to Leptos with Tailwind"</h2>
            <p class="bg-white px-10 py-10 text-black rounded-lg">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
            <button
                class="m-8 bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                //class="m-10 btn btn-active btn-accent"
                on:click=move |_| set_count.update(|count| *count += 1)
            >
                "Something's here | "
                {
                    move || if count() == 0 {
                        "Click me!".to_string()
                    } else {
                        count().to_string()
                    }
                }
                " | Some more text"
            </button>
        </div>
    }
}
