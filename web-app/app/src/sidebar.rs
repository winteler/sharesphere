use leptos::prelude::*;

use crate::app::GlobalState;
use crate::constants::PATH_SEPARATOR;
use crate::forum::{get_popular_forum_headers, get_subscribed_forum_headers, ForumHeader, ForumState, FORUM_ROUTE_PREFIX};
use crate::forum_category::ForumCategoryBadge;
use crate::unpack::TransitionUnpack;
use crate::widget::{Collapse, TitleCollapse};

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
        <TitleCollapse title=title>
            <ul class="flex flex-col pt-1 pl-1">
            {
                forum_header_vec.iter().map(|forum_header| {
                    let forum_path = FORUM_ROUTE_PREFIX.to_owned() + PATH_SEPARATOR + &forum_header.forum_name;
                    view! {
                        <li class="px-2 rounded hover:bg-base-content/20">
                            <a href=forum_path>
                                <ForumHeader forum_header=forum_header.clone()/>
                            </a>
                        </li>
                    }
                }).collect_view()
            }
            </ul>
        </TitleCollapse>
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
        <div class="flex flex-col justify-start w-60 h-full pl-2 pt-2 max-2xl:bg-base-300">
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
                <TransitionUnpack resource=forum_state.forum_resource let:forum>
                    <div class="pl-4 whitespace-pre-wrap">{forum.description.clone()}</div>
                </TransitionUnpack>
            </div>
            <div class="border-b border-primary/80"/>
            <ForumRuleList/>
            <div class="border-b border-primary/80"/>
            <ForumCategoryList/>
            <div class="border-b border-primary/80"/>
            <ModeratorList/>
        </div>
    }
}

/// List of moderators for a forum
#[component]
pub fn ForumRuleList() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <TitleCollapse title="Rules">
            <div class="flex flex-col pl-2 pt-1">
                <TransitionUnpack resource=forum_state.forum_rules_resource let:forum_rule_vec>
                {
                    let mut index = 0usize;
                    forum_rule_vec.iter().map(|rule| {
                        let description = StoredValue::new(rule.description.clone());
                        let title = rule.title.clone();
                        let title_view = move || view! {
                            <div class="flex gap-4 items-center">
                                <div>{index}</div>
                                <div>{title}</div>
                            </div>
                        };
                        index += 1;
                        view! {
                            <Collapse
                                title_view
                                is_open=false
                            >
                                <div class="pl-2 text-sm">{description.get_value()}</div>
                            </Collapse>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
        </TitleCollapse>
    }
}

/// List of categories for a forum
#[component]
pub fn ForumCategoryList() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
        <TitleCollapse title="Categories">
            <div class="flex flex-col pl-2 pt-1">
                <TransitionUnpack resource=forum_state.forum_categories_resource let:forum_category_vec>
                {
                    forum_category_vec.iter().map(|forum_category| {
                        let category_header = forum_category.clone().into();
                        let description = StoredValue::new(forum_category.description.clone());
                        view! {
                            <Collapse
                                title_view=move || view! { <ForumCategoryBadge category_header/> }
                                is_open=false
                            >
                                <div class="pl-2 text-sm">{description.get_value()}</div>
                            </Collapse>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
        </TitleCollapse>
    }
}

/// List of moderators for a forum
#[component]
pub fn ModeratorList() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    view! {
         <TitleCollapse title="Moderators">
            <div class="flex flex-col gap-1">
                <div class="flex border-b border-base-content/20 pl-4">
                    <div class="w-1/2 py-2 text-left font-semibold">Username</div>
                    <div class="w-1/2 py-2 text-left font-semibold">Role</div>
                </div>
                <TransitionUnpack resource=forum_state.forum_roles_resource let:forum_role_vec>
                {
                    forum_role_vec.iter().map(|role| {
                        view! {
                            <div class="flex py-1 pl-4">
                                <div class="w-1/2 select-none">{role.username.clone()}</div>
                                <div class="w-1/2 select-none">{role.permission_level.to_string()}</div>
                            </div>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
        </TitleCollapse>
    }
}
