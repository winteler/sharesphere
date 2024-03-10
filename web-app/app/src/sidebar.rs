use leptos::*;

use crate::app::GlobalState;
use crate::forum::{FORUM_ROUTE_PREFIX, get_popular_forum_names, get_subscribed_forum_names};
use crate::icons::{ErrorIcon, LoadingIcon};

/// Component to display a list of forum links
#[component]
pub fn ForumLinkList<'a>(
    title: &'static str,
    forum_name_vec: &'a Vec<String>,
) -> impl IntoView {
    let forum_vector_view = forum_name_vec.iter().map(|forum_name| {
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
        <ul class="menu h-full">
            <li>
                <details open>
                    <summary class="text-xl font-medium">{title}</summary>
                    <ul class="menu-dropdown">
                        {forum_vector_view}
                    </ul>
                </details>
            </li>
        </ul>
    }
}

/// Left sidebar component
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let subscribed_forum_vec = create_resource(
        move || (state.subscribe_action.version().get(), state.unsubscribe_action.version().get()),
        |_| get_subscribed_forum_names()
    );
    let popular_forum_vec = create_resource(
        move || state.create_forum_action.version().get(),
        |_| get_popular_forum_names()
    );

    view! {
        <div class="flex flex-col justify-start w-60 h-full max-2xl:bg-base-200">
            <div>
                <Transition fallback=move || view! {  <LoadingIcon/> }>
                    { move || {
                        subscribed_forum_vec.map(|subscribed_forum_vec| match subscribed_forum_vec {
                            Ok(subscribed_forum_vec) => {
                                view! {
                                    <ForumLinkList
                                        title="Subscribed"
                                        forum_name_vec=subscribed_forum_vec
                                    />
                                }.into_view()
                            },
                            Err(e) => {
                                log::trace!("Error: {}", e);
                                View::default()
                            }
                        })
                    }}
                </Transition>
            </div>
            <div>
                <Transition fallback=move || view! {  <LoadingIcon/> }>
                    { move || {
                         popular_forum_vec.map(|popular_forum_vec| match popular_forum_vec {
                            Ok(popular_forum_vec) => {
                                view! {
                                    <ForumLinkList
                                        title="Popular"
                                        forum_name_vec=popular_forum_vec
                                    />
                                }.into_view()
                            },
                            Err(e) => {
                                log::error!("Error loading popular forums: {}", e);
                                view! { <ErrorIcon/> }.into_view()
                            }
                        })
                    }}
                </Transition>
            </div>
        </div>
    }
}

/// Right sidebar component
#[component]
pub fn RightSidebar() -> impl IntoView {
    view! {
        <div class="h-full p-4 w-40"></div>
    }
}