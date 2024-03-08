use leptos::*;

use crate::app::GlobalState;
use crate::forum::{FORUM_ROUTE_PREFIX, get_subscribed_forums};
use crate::icons::{ErrorIcon, LoadingIcon};

/// Left sidebar component
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let subscribed_forum_set = create_resource(
        move || (state.subscribe_action.version().get(), state.unsubscribe_action.version().get()),
        |_| get_subscribed_forums()
    );

    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
            { move || {
                     subscribed_forum_set.map(|subscribed_forum_set| match subscribed_forum_set {
                        Ok(subscribed_forum_vec) => {
                            let forum_vector_view = subscribed_forum_vec.iter().map(|forum_name| {
                                let forum_path = FORUM_ROUTE_PREFIX.to_owned() + "/" + forum_name;
                                view! {
                                    <li>
                                        <a href=forum_path>
                                            {forum_name}
                                        </a>
                                    </li>
                                }
                            }).collect_view();
                            view! {
                                <ul class="menu h-full p-4 w-40 max-2xl:bg-base-200">
                                    {forum_vector_view}
                                </ul>
                            }.into_view()
                        },
                        Err(e) => {
                            log::info!("Error: {}", e);
                            view! { <ErrorIcon/> }.into_view()
                        }
                    })
                }
            }
        </Transition>
    }
}

/// Right sidebar component
#[component]
pub fn RightSidebar() -> impl IntoView {
    view! {
        <div class="h-full p-4 w-40"></div>
    }
}