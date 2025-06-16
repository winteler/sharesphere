use leptos::prelude::*;
use leptos::html::Div;
#[cfg(feature = "hydrate")]
use leptos_use::on_click_outside;
use sharesphere_utils::errors::{AppError};
use sharesphere_utils::routes::{ABOUT_SHARESPHERE_ROUTE, CONTENT_POLICY_ROUTE, PRIVACY_POLICY_ROUTE, RULES_ROUTE, TERMS_AND_CONDITIONS_ROUTE};
use sharesphere_utils::unpack::{TransitionUnpack};
use sharesphere_utils::widget::{Collapse, ContentBody, TitleCollapse};
use crate::rule::{get_rule_vec, Rule};
use crate::sphere::{SphereHeader, SphereLinkList};

use crate::state::{GlobalState, SphereState};
use crate::search::{SearchSpheres, SearchState};
use crate::sphere::{get_popular_sphere_headers, get_subscribed_sphere_headers};
use crate::sphere_category::SphereCategoryBadge;

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

    let sidebar_class = move || match state.show_left_sidebar.get() {
        true => "left_sidebar_base_class max-2xl:translate-x-0 transition-transform duration-300 ease-in-out",
        false => "left_sidebar_base_class max-2xl:-translate-x-100 transition-transform duration-300 ease-in-out",
    };
    let sidebar_ref = NodeRef::<Div>::new();
    #[cfg(feature = "hydrate")]
    {
        // only enable with "hydrate" to avoid server side "Dropped SendWrapper" error
        let _ = on_click_outside(sidebar_ref, move |_| state.show_left_sidebar.set(false));
    }

    view! {
        <div class=sidebar_class node_ref=sidebar_ref>
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
        <Show when=state.show_left_sidebar>
            <div class="absolute top-0 right-0 h-full w-full bg-base-200/50"/>
        </Show>
    }
}

/// Home right sidebar component
#[component]
pub fn HomeSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sidebar_class = move || match state.show_right_sidebar.get() {
        true => "right_sidebar_base_class max-2xl:translate-x-0 transition-transform duration-300 ease-in-out",
        false => "right_sidebar_base_class max-2xl:translate-x-100 transition-transform duration-300 ease-in-out",
    };
    let sidebar_ref = NodeRef::<Div>::new();
    #[cfg(feature = "hydrate")]
    {
        // only enable with "hydrate" to avoid server side "Dropped SendWrapper" error
        let _ = on_click_outside(sidebar_ref, move |_| state.show_right_sidebar.set(false));
    }
    let rule_resource = Resource::new(
        || (),
        |_| get_rule_vec(None)
    );

    view! {
        <div class=sidebar_class node_ref=sidebar_ref>
            <div class="flex flex-col gap-2">
                <h1 class="text-2xl font-semibold text-center">"Welcome to ShareSphere!"</h1>
                <div class="flex flex-col gap-2">
                    <p class="text-justify">
                        "ShareSphere is the place to exchange with other people about your hobbies, news, art, jokes and many more topics."
                    </p>
                    <p class="text-justify">
                        "ShareSphere is a non-profit, add-free, open source website with a focus on transparency, privacy and community empowerment. \
                        You can find more information on the website and its rules below."
                    </p>
                </div>
                <ul class="list-disc list-inside">
                    <li><a href=ABOUT_SHARESPHERE_ROUTE class="link text-primary">"About ShareSphere"</a></li>
                    <li><a href=TERMS_AND_CONDITIONS_ROUTE class="link text-primary">"Terms and conditions"</a></li>
                    <li><a href=PRIVACY_POLICY_ROUTE class="link text-primary">"Privacy Policy"</a></li>
                    <li><a href=CONTENT_POLICY_ROUTE class="link text-primary">"Content Policy"</a></li>
                    <li><a href=RULES_ROUTE class="link text-primary">"Rules"</a></li>
                </ul>
                <RuleList rule_resource=rule_resource.into()/>
            </div>
        </div>
        <Show when=state.show_right_sidebar>
            <div class="absolute top-0 left-0 h-full w-full bg-base-200/50"/>
        </Show>
    }
}

/// Sphere right sidebar component
#[component]
pub fn SphereSidebar() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();

    let sidebar_class = move || match state.show_right_sidebar.get() {
        true => "right_sidebar_base_class max-2xl:translate-x-0 transition-transform duration-300 ease-in-out",
        false => "right_sidebar_base_class max-2xl:translate-x-100 transition-transform duration-300 ease-in-out",
    };
    let sidebar_ref = NodeRef::<Div>::new();
    #[cfg(feature = "hydrate")]
    {
        // only enable with "hydrate" to avoid server side "Dropped SendWrapper" error
        let _ = on_click_outside(sidebar_ref, move |_| state.show_right_sidebar.set(false));
    }
    view! {
        <div class=sidebar_class node_ref=sidebar_ref>
            <div class="flex flex-col gap-2">
                <div class="text-2xl font-semibold text-center">{sphere_state.sphere_name}</div>
                <TransitionUnpack resource=sphere_state.sphere_resource let:sphere>
                    <div class="pl-4 whitespace-pre-wrap">{sphere.description.clone()}</div>
                </TransitionUnpack>
            </div>
            <div class="border-b border-primary/80"/>
            <RuleList rule_resource=sphere_state.sphere_rules_resource/>
            <div class="border-b border-primary/80"/>
            <SphereCategoryList/>
            <div class="border-b border-primary/80"/>
            <ModeratorList/>
        </div>
        <Show when=state.show_right_sidebar>
            <div class="absolute top-0 left-0 h-full w-full bg-base-200/50"/>
        </Show>
    }
}

/// List of rules given in the input resource
#[component]
fn RuleList(
    rule_resource: Resource<Result<Vec<Rule>, AppError>>
) -> impl IntoView {
    view! {
        <TitleCollapse title="Rules">
            <div class="flex flex-col pl-2 pt-1 gap-1">
                <TransitionUnpack resource=rule_resource let:rule_vec>
                {
                    rule_vec.iter().enumerate().map(|(index, rule)| {
                        let description = StoredValue::new(rule.description.clone());
                        let is_markdown = rule.markdown_description.is_some();
                        let title = rule.title.clone();
                        let title_view = move || view! {
                            <div class="flex gap-2">
                                <div>{index+1}</div>
                                <div class="text-left">{title}</div>
                            </div>
                        };
                        view! {
                            <Collapse
                                title_view
                                is_open=false
                            >
                                <ContentBody body=description.get_value() is_markdown/>
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
