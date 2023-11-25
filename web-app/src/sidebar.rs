use leptos::*;

use crate::app::GlobalState;
use crate::forum::{FORUM_ROUTE_PREFIX, get_subscribed_forums};
use crate::icons::{ ErrorIcon, LoadingIcon};

/// Left sidebar component
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let subscribed_forum_set = create_resource(move || (state.create_forum_action.version().get()), |_| get_subscribed_forums());

    view! {
        <ul class="menu h-full p-4 w-40 max-2xl:bg-base-200">
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                { move || {
                         subscribed_forum_set.with(|subscribed_forum_set| match subscribed_forum_set {
                            Some(Ok(subscribed_forum_set)) => {
                                subscribed_forum_set.iter().map(|forum_name| {
                                    let forum_path = FORUM_ROUTE_PREFIX.to_owned() + "/" + forum_name;
                                    view! {
                                        <li>
                                            <a href=forum_path>
                                                {forum_name}
                                            </a>
                                        </li>
                                    }
                                }).collect_view()
                            },
                            Some(Err(e)) => {
                                log::info!("Error: {}", e);
                                view! { <ErrorIcon/> }.into_view()
                            },
                            None => {
                                log::trace!("Resource not loaded yet.");
                                view! { <ErrorIcon/> }.into_view()
                            }
                        })
                    }
                }
            </Transition>
        </ul>
    }
}

/// Right sidebar component
#[component]
pub fn RightSidebar() -> impl IntoView {
    view! {
        <div class="h-full p-4 w-40"></div>
    }
}