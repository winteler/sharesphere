use leptos::*;
use std::collections::BTreeSet;

use crate::forum::{FORUM_ROUTE_PREFIX, get_subscribed_forums};
use crate::icons::{ ErrorIcon, LoadingIcon};

/// Navigation bar component
#[component]
pub fn Drawer() -> impl IntoView {

    let subscribed_forum_set = create_resource(|| (), |_| get_subscribed_forums());

    view! {
        <label for="my-drawer" class="drawer-overlay"></label>
        <ul class="menu p-4 w-40 h-full bg-base-200 text-base-content">
            <Transition fallback=move || view! {  <LoadingIcon/> }>
                { move || {
                         subscribed_forum_set.get().map(|subscribed_forum_set: Result<BTreeSet<String>, ServerFnError>| match subscribed_forum_set {
                            Err(e) => {
                                log::info!("Error: {}", e);
                                view! { <ErrorIcon/> }.into_view()
                            },
                            Ok(subscribed_forum_set) => {
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
                        })
                    }
                }
            </Transition>
        </ul>
    }
}