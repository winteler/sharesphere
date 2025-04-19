use leptos::prelude::*;
use server_fn::const_format::concatcp;
use sharesphere_utils::unpack::TransitionUnpack;
use sharesphere_utils::widget::{Collapse, TitleCollapse};

use crate::sphere::{SphereHeader, SphereLinkList};

use crate::state::{GlobalState, SphereState};
use crate::search::{SearchSpheres, SearchState};
use crate::sphere::{get_popular_sphere_headers, get_subscribed_sphere_headers};
use crate::sphere_category::SphereCategoryBadge;

const BASE_RIGHT_SIDEBAR_CLASS: &'static str = "flex flex-col justify-start w-80 h-full px-3 py-2 bg-base-200 2xl:bg-base-100 max-2xl:absolute max-2xl:top-0 max-2xl:right-0 ";

/// Component to display a collapsable list of sphere links
#[component]
pub fn SphereLinkListCollapse(
    title: &'static str,
    sphere_header_vec: Vec<SphereHeader>,
    #[prop(default = true)]
    is_open: bool,
) -> impl IntoView {
    if sphere_header_vec.is_empty() {
        return ().into_any()
    }
    view! {
        <TitleCollapse title=title is_open>
            <SphereLinkList sphere_header_vec=sphere_header_vec.clone()/>
        </TitleCollapse>
    }.into_any()
}

/// Left sidebar component
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let search_state = SearchState::default();
    let subscribed_sphere_vec_resource = Resource::new(
        move || {
            (
                state.logout_action.version().get(),
                state.create_sphere_action.version().get(),
                state.sphere_reload_signal.get(),
                state.subscribe_action.version().get(),
                state.unsubscribe_action.version().get(),
            )
        },
        |_| get_subscribed_sphere_headers(),
    );
    let popular_sphere_vec_resource = Resource::new(
        move || state.sphere_reload_signal.get(),
        |_| get_popular_sphere_headers(),
    );

    view! {
        <div class="flex flex-col justify-start w-60 h-full h-min-0 overflow-y-auto p-2 max-2xl:bg-base-300">
            <TransitionUnpack resource=subscribed_sphere_vec_resource let:sphere_header_vec>
                <SphereLinkListCollapse
                    title="Subscribed"
                    sphere_header_vec=sphere_header_vec.clone()
                />
            </TransitionUnpack>
            <TransitionUnpack resource=popular_sphere_vec_resource let:popular_sphere_header_vec>
                <SphereLinkListCollapse
                    title="Popular"
                    sphere_header_vec=popular_sphere_header_vec.clone()
                    is_open=false
                />
            </TransitionUnpack>
            <div class="flex flex-col gap-2 pt-4 max-h-124">
                <SearchSpheres search_state class="w-full gap-2" autofocus=false/>
            </div>
        </div>
    }
}

/// Home right sidebar component
#[component]
pub fn HomeSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sidebar_class = move || match state.show_right_sidebar.get() {
        true => concatcp!(BASE_RIGHT_SIDEBAR_CLASS, "max-2xl:translate-x-0 transition-transform duration-250 ease-in-out"),
        false => concatcp!(BASE_RIGHT_SIDEBAR_CLASS, "max-2xl:translate-x-100 transition-transform duration-250 ease-in-out"),
    };
    view! {
        <div class=sidebar_class>
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

/// Sphere right sidebar component
#[component]
pub fn SphereSidebar() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    view! {
        <div class="flex flex-col gap-2 justify-start w-80 h-full px-4 py-2">
            <div class="flex flex-col gap-2">
                <div class="text-2xl font-semibold text-center">{sphere_state.sphere_name}</div>
                <TransitionUnpack resource=sphere_state.sphere_resource let:sphere>
                    <div class="pl-4 whitespace-pre-wrap">{sphere.description.clone()}</div>
                </TransitionUnpack>
            </div>
            <div class="border-b border-primary/80"/>
            <SphereRuleList/>
            <div class="border-b border-primary/80"/>
            <SphereCategoryList/>
            <div class="border-b border-primary/80"/>
            <ModeratorList/>
        </div>
    }
}

/// List of moderators for a sphere
#[component]
pub fn SphereRuleList() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    view! {
        <TitleCollapse title="Rules">
            <div class="flex flex-col pl-2 pt-1">
                <TransitionUnpack resource=sphere_state.sphere_rules_resource let:sphere_rule_vec>
                {
                    let mut index = 0usize;
                    sphere_rule_vec.iter().map(|rule| {
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

/// List of categories for a sphere
#[component]
pub fn SphereCategoryList() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    view! {
        <TitleCollapse title="Categories">
            <div class="flex flex-col pl-2 pt-1">
                <TransitionUnpack resource=sphere_state.sphere_categories_resource let:sphere_category_vec>
                {
                    sphere_category_vec.iter().map(|sphere_category| {
                        let category_header = sphere_category.clone().into();
                        let description = StoredValue::new(sphere_category.description.clone());
                        view! {
                            <Collapse
                                title_view=move || view! { <SphereCategoryBadge category_header/> }
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

/// List of moderators for a sphere
#[component]
pub fn ModeratorList() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    view! {
         <TitleCollapse title="Moderators">
            <div class="flex flex-col gap-1">
                <div class="flex border-b border-base-content/20 pl-4">
                    <div class="w-1/2 py-2 text-left font-semibold">Username</div>
                    <div class="w-1/2 py-2 text-left font-semibold">Role</div>
                </div>
                <TransitionUnpack resource=sphere_state.sphere_roles_resource let:sphere_role_vec>
                {
                    sphere_role_vec.iter().map(|role| {
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
