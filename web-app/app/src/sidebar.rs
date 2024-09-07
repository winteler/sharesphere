use leptos::*;

use crate::app::GlobalState;
use crate::forum::{get_popular_forum_names, get_subscribed_forum_names, ForumState, FORUM_ROUTE_PREFIX};
use crate::unpack::TransitionUnpack;
use crate::widget::MinimizeMaximizeWidget;

/// Component to display a list of forum links
#[component]
pub fn ForumLinkList(title: &'static str, forum_name_vec: Vec<String>) -> impl IntoView {
    if forum_name_vec.is_empty() {
        return View::default()
    }

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
        <div class="flex flex-col justify-start w-60 h-full max-2xl:bg-base-300">
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
        <div class="flex flex-col gap-4 justify-start w-80 h-full px-4 py-2">
            <div class="flex flex-col gap-2">
                <div class="text-2xl text-center">{forum_state.forum_name}</div>
                <TransitionUnpack resource=forum_state.forum_resource let:forum>
                    <div class="pl-4 whitespace-pre-wrap">{forum.description.clone()}</div>
                </TransitionUnpack>
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
        <div class="flex flex-col gap-1">
            <div class="text-xl text-center">"Rules"</div>
            <TransitionUnpack resource=forum_state.forum_rules_resource let:forum_rule_vec>
            {
                let forum_rule_vec = forum_rule_vec.clone();
                view! {
                    <div class="flex flex-col gap-1 pl-4">
                        <For
                            each= move || forum_rule_vec.clone().into_iter().enumerate()
                            key=|(_index, rule)| (rule.rule_id)
                            children=move |(_, rule)| {
                                let show_description = create_rw_signal(false);
                                let description = store_value(rule.description);
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
                                            <div class="text-l font-bold">{rule.title}</div>
                                            <MinimizeMaximizeWidget is_maximized=show_description/>
                                        </div>
                                        <div class=class>
                                            {description.get_value()}
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                }
            }
            </TransitionUnpack>
        </div>
    }
}

/// List of moderators for a forum
#[component]
pub fn ModeratorList() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <div class="flex flex-col gap-1">
            <div class="text-xl text-center">"Moderators"</div>
            <TransitionUnpack resource=forum_state.forum_roles_resource let:forum_role_vec>
            {
                let forum_role_vec = forum_role_vec.clone();
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex border-b border-base-content/20 pl-4">
                            <div class="w-1/2 py-2 text-left font-bold">Username</div>
                            <div class="w-1/2 py-2 text-left font-bold">Role</div>
                        </div>
                        <For
                            each= move || forum_role_vec.clone().into_iter().enumerate()
                            key=|(_index, role)| (role.user_id, role.permission_level)
                            children=move |(_, role)| {
                                let username = store_value(role.username);
                                view! {
                                    <div class="flex py-1 rounded hover:bg-base-content/20 pl-4">
                                        <div class="w-1/2 select-none">{username.get_value()}</div>
                                        <div class="w-1/2 select-none">{role.permission_level.to_string()}</div>
                                    </div>
                                }
                            }
                        />
                    </div>
                }
            }
            </TransitionUnpack>
        </div>
    }
}
