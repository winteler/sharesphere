use leptos::prelude::*;

use crate::app::GlobalState;
use crate::constants::PATH_SEPARATOR;
use crate::forum::{get_popular_forum_headers, get_subscribed_forum_headers, ForumHeader, ForumState, FORUM_ROUTE_PREFIX};
use crate::unpack::{ArcTransitionUnpack, TransitionUnpack};
use crate::widget::MinimizeMaximizeWidget;

/// Component to display a list of forum links
#[component]
pub fn ForumLinkList(
    title: &'static str,
    forum_header_vec: Vec<ForumHeader>
) -> impl IntoView {
    if forum_header_vec.is_empty() {
        return ().into_any()
    }
    view! {
        <details class="collapse collapse-arrow menu" open>
            <summary class="collapse-title text-xl font-medium rounded-md hover:bg-base-content/20">{title}</summary>
            <ul class="collapse-content menu-dropdown">
            {
                forum_header_vec.into_iter().map(|forum_header| {
                    let forum_path = FORUM_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + &forum_header.forum_name;
                    view! {
                        <li>
                            <a href=forum_path>
                                <ForumHeader forum_header/>
                            </a>
                        </li>
                    }
                }).collect_view()
            }
            </ul>
        </details>
    }.into_any()
}

/// Left sidebar component
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let subscribed_forum_vec_resource = Resource::new(
        move || {
            (
                state.logout_action.version().get(),
                state.create_forum_action.version().get(),
                state.forum_reload_signal.get(),
                state.subscribe_action.version().get(),
                state.unsubscribe_action.version().get(),
            )
        },
        |_| get_subscribed_forum_headers(),
    );
    let popular_forum_vec_resource = Resource::new(
        move || state.forum_reload_signal.get(),
        |_| get_popular_forum_headers(),
    );

    view! {
        <div class="flex flex-col justify-start w-60 h-full max-2xl:bg-base-300">
            <div>
                <TransitionUnpack resource=subscribed_forum_vec_resource let:forum_header_vec>
                    <ForumLinkList
                        title="Subscribed"
                        forum_header_vec=forum_header_vec.clone()
                    />
                </TransitionUnpack>
            </div>
            <div>
                <TransitionUnpack resource=popular_forum_vec_resource let:forum_header_vec>
                    <ForumLinkList
                        title="Popular"
                        forum_header_vec=forum_header_vec.clone()
                    />
                </TransitionUnpack>
            </div>
        </div>
    }
}

/// Home right sidebar component
#[component]
pub fn HomeSidebar() -> impl IntoView {
    view! {
        <div class="flex flex-col justify-start w-80 h-full px-3 py-2">
            <div class="flex flex-col gap-2">
                <div class="text-2xl text-center">"Welcome to ShareSphere!"</div>
                <div class="flex flex-col gap-1 text-justify">
                    <p>"ShareSphere is the place to exchange with other people about your hobbies, news, art, jokes and many more topics."</p>
                    <p>"ShareSphere is a non-profit, open source website. You can find more information on the website and its rules below."</p>
                </div>
            </div>
        </div>
    }
}

/// Forum right sidebar component
#[component]
pub fn ForumSidebar() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <div class="flex flex-col gap-2 justify-start w-80 h-full px-4 py-2">
            <div class="flex flex-col gap-2">
                <div class="text-2xl font-semibold text-center">{forum_state.forum_name}</div>
                <ArcTransitionUnpack resource=forum_state.forum_resource let:forum>
                    <div class="pl-4 whitespace-pre-wrap">{forum.description.clone()}</div>
                </ArcTransitionUnpack>
            </div>
            <ForumRuleList/>
            <ModeratorList/>
        </div>
    }
}

/// List of moderators for a forum
#[component]
pub fn ForumRuleList() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <details class="collapse collapse-arrow rounded-md" open>
            <summary class="collapse-title text-xl font-semibold rounded-md hover:bg-base-content/20">"Rules"</summary>
            <div class="collapse-content flex flex-col gap-1 pl-4 pt-1">
                <TransitionUnpack resource=forum_state.forum_rules_resource let:forum_rule_vec>
                {
                    forum_rule_vec.iter().map(|rule| {
                        let show_description = RwSignal::new(false);
                        let description = StoredValue::new(rule.description.clone());
                        let class = move || match show_description.get() {
                            true => "transition duration-500 opacity-100 visible",
                            false => "transition duration-500 opacity-0 invisible h-0",
                        };
                        view! {
                            <div class="flex flex-col gap-1">
                                <div
                                    class="flex justify-between"
                                    on:click=move |_| show_description.update(|value| *value = !*value)
                                >
                                    <div class="text-l font-semibold">{rule.title.clone()}</div>
                                    <MinimizeMaximizeWidget is_maximized=show_description/>
                                </div>
                                <div class=class>
                                    {description.get_value()}
                                </div>
                            </div>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
        </details>
    }
}

/// List of moderators for a forum
#[component]
pub fn ModeratorList() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <details class="collapse collapse-arrow rounded-md" open>
            <summary class="collapse-title text-xl font-semibold rounded-md hover:bg-base-content/20">"Moderators"</summary>
            <div class="flex flex-col gap-1">
                <div class="flex border-b border-base-content/20 pl-4">
                    <div class="w-1/2 py-2 text-left font-semibold">Username</div>
                    <div class="w-1/2 py-2 text-left font-semibold">Role</div>
                </div>
                <TransitionUnpack resource=forum_state.forum_roles_resource let:forum_role_vec>
                {
                    forum_role_vec.iter().map(|role| {
                        view! {
                            <div class="flex py-1 rounded hover:bg-base-content/20 pl-4">
                                <div class="w-1/2 select-none">{role.username.clone()}</div>
                                <div class="w-1/2 select-none">{role.permission_level.to_string()}</div>
                            </div>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
        </details>
    }
}

/// List of categories for a forum
#[component]
pub fn CategoryList() -> impl IntoView {
    view! {
        
    }
}