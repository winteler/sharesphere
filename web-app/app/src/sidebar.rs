use leptos::*;

use crate::app::GlobalState;
use crate::forum::{get_popular_forum_names, get_subscribed_forum_names, FORUM_ROUTE_PREFIX};
use crate::unpack::TransitionUnpack;

/// Component to display a list of forum links
#[component]
pub fn ForumLinkList(title: &'static str, forum_name_vec: Vec<String>) -> impl IntoView {
    if forum_name_vec.is_empty() {
        View::default()
    } else {
        view! {
            <ul class="menu h-full">
                <li>
                    <details open>
                        <summary class="text-xl font-medium">{title}</summary>
                        <ul class="menu-dropdown">
                            <For
                                // a function that returns the items we're iterating over; a signal is fine
                                each=move || forum_name_vec.clone().into_iter().enumerate()
                                // a unique key for each item as a reference
                                key=|(_, forum_name)| forum_name.clone()
                                // renders each item to a view
                                children=move |(_, forum_name)| {
                                    let forum_path = FORUM_ROUTE_PREFIX.to_owned() + "/" + &forum_name;
                                    view! {
                                        <li>
                                            <a href=forum_path>
                                                {forum_name}
                                            </a>
                                        </li>
                                    }
                                }
                            />
                        </ul>
                    </details>
                </li>
            </ul>
        }.into_view()
    }
}

/// Left sidebar component
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let subscribed_forum_vec_resource = create_resource(
        move || {
            (
                state.subscribe_action.version().get(),
                state.unsubscribe_action.version().get(),
            )
        },
        |_| get_subscribed_forum_names(),
    );
    let popular_forum_vec_resource = create_resource(
        move || (),
        |_| get_popular_forum_names(),
    );

    view! {
        <div class="flex flex-col justify-start w-60 h-full max-2xl:bg-base-200">
            <div>
                <TransitionUnpack resource=subscribed_forum_vec_resource let:forum_vec>
                    <ForumLinkList
                        title="Subscribed"
                        forum_name_vec=forum_vec.clone()
                    />
                </TransitionUnpack>
            </div>
            <div>
                <TransitionUnpack resource=popular_forum_vec_resource let:forum_vec>
                    <ForumLinkList
                        title="Popular"
                        forum_name_vec=forum_vec.clone()
                    />
                </TransitionUnpack>
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
