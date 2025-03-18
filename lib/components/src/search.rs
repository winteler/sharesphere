use leptos::html;
use leptos::prelude::*;
use leptos_router::components::Form;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};
use sharesphere_core::comment::CommentMiniatureList;
use sharesphere_core::post::PostMiniatureList;
use sharesphere_core::search::{get_matching_user_header_vec, search_comments, search_posts, SearchForm, SearchSpheres, SearchState};
use sharesphere_core::sidebar::HomeSidebar;
use sharesphere_core::state::SphereState;
use sharesphere_utils::icons::MagnifierIcon;
use sharesphere_utils::routes::{SEARCH_ROUTE, SEARCH_TAB_QUERY_PARAM};
use sharesphere_utils::unpack::{handle_additional_load, handle_initial_load, TransitionUnpack};
use sharesphere_utils::widget::{EnumQueryTabs, ToView};
use crate::profile::UserHeaderLink;

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SearchType {
    #[default]
    Spheres,
    Posts,
    Comments,
    Users,
}

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SphereSearchType {
    #[default]
    Posts,
    Comments,
}

impl ToView for SearchType {
    fn to_view(self) -> AnyView {
        match self {
            SearchType::Spheres => view! { <SearchSpheresWithContext/> }.into_any(),
            SearchType::Posts => view! { <SearchPosts/> }.into_any(),
            SearchType::Comments => view! { <SearchComments/> }.into_any(),
            SearchType::Users => view! { <SearchUsers/> }.into_any(),
        }
    }
}

impl ToView for SphereSearchType {
    fn to_view(self) -> AnyView {
        match self {
            SphereSearchType::Posts => view! { <SearchPosts/> }.into_any(),
            SphereSearchType::Comments => view! { <SearchComments/> }.into_any(),
        }
    }
}

/// Button to navigate to the search page
#[component]
pub fn SearchButton() -> impl IntoView
{
    let tab: &'static str = SearchType::default().into();
    view! {
        <Form method="GET" action=SEARCH_ROUTE>
            <input name=SEARCH_TAB_QUERY_PARAM value=tab class="hidden"/>
            <button class="btn btn-ghost btn-circle">
                <MagnifierIcon/>
            </button>
        </Form>
    }
}

/// Component to search spheres, posts, comments and users
#[component]
pub fn Search() -> impl IntoView
{
    provide_context(SearchState::default());
    view! {
        <div class="w-full flex justify-center">
            <div class="w-full 2xl:w-2/3 flex flex-col">
                <EnumQueryTabs
                    query_param=SEARCH_TAB_QUERY_PARAM
                    query_enum_iter=SearchType::iter()
                />
            </div>
        </div>
        <div class="max-2xl:hidden">
            <HomeSidebar/>
        </div>
    }
}

/// Component to search posts, comments in a sphere
#[component]
pub fn SphereSearch() -> impl IntoView
{
    provide_context(SearchState::default());
    view! {
        <div class="w-full flex justify-center">
            <div class="w-full flex flex-col">
                <EnumQueryTabs
                    query_param=SEARCH_TAB_QUERY_PARAM
                    query_enum_iter=SphereSearchType::iter()
                />
            </div>
        </div>
    }
}

/// Component to search spheres, uses the SearchState from the context to get user input
#[component]
pub fn SearchSpheresWithContext() -> impl IntoView
{
    let search_state = expect_context::<SearchState>();
    view! {
        <SearchSpheres search_state/>
    }
}

#[component]
pub fn SearchPosts() -> impl IntoView
{
    let search_state = expect_context::<SearchState>();
    let sphere_state = use_context::<SphereState>();

    let post_vec = RwSignal::new(Vec::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_post_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let search_input = search_state.search_input_debounced.get();
            let initial_load = match search_input.is_empty() {
                true => Ok(Vec::new()),
                false => search_posts(
                    search_state.search_input_debounced.get(),
                    sphere_state.map(|sphere_state| sphere_state.sphere_name.get()),
                    search_state.show_spoiler.get(),
                    0,
                ).await,
            };
            handle_initial_load(initial_load, post_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let additional_load = search_posts(
                    search_state.search_input_debounced.get(),
                    sphere_state.map(|sphere_state| sphere_state.sphere_name.get()),
                    search_state.show_spoiler.get(),
                    post_vec.read_untracked().len(),
                ).await;
                handle_additional_load(additional_load, post_vec, load_error);
                is_loading.set(false);
            }
        }
    );
    view! {
        <SearchForm
            search_state
            show_spoiler_checkbox=true
        />
        <PostMiniatureList
            post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}

#[component]
pub fn SearchComments() -> impl IntoView
{
    let search_state = expect_context::<SearchState>();
    let sphere_state = use_context::<SphereState>();

    let comment_vec = RwSignal::new(Vec::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_comment_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let search_input = search_state.search_input_debounced.get();
            let initial_load = match search_input.is_empty() {
                true => Ok(Vec::new()),
                false => search_comments(
                    search_state.search_input_debounced.get(),
                    sphere_state.map(|sphere_state| sphere_state.sphere_name.get()),
                    0,
                ).await,
            };
            handle_initial_load(initial_load, comment_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_comment_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let additional_load = search_comments(
                    search_state.search_input_debounced.get_untracked(),
                    sphere_state.map(|sphere_state| sphere_state.sphere_name.get()),
                    comment_vec.read_untracked().len()
                ).await;
                handle_additional_load(additional_load, comment_vec, load_error);
                is_loading.set(false);
            }
        }
    );
    view! {
        <SearchForm
            search_state
            show_spoiler_checkbox=false
        />
        <CommentMiniatureList
            comment_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}

#[component]
pub fn SearchUsers() -> impl IntoView
{
    let search_state = expect_context::<SearchState>();
    let search_user_resource = Resource::new(
        move || search_state.search_input_debounced.get(),
        move |search_input| async move {
            match search_input.is_empty() {
                true => Ok(Vec::new()),
                false => get_matching_user_header_vec(search_input, None, 50).await,
            }
        }
    );
    view! {
        <SearchForm
            search_state
            show_spoiler_checkbox=false
        />
        <TransitionUnpack resource=search_user_resource let:user_header_vec>
        { match user_header_vec.is_empty() {
            true => None,
            false => {
                let user_header_link_list = user_header_vec.iter().map(|user_header| view! {
                    <UserHeaderLink user_header/>
                }).collect_view();
                Some(view! {
                    <div class="flex flex-col gap-2 self-center p-2 bg-base-200 rounded-sm overflow-y-auto max-h-full w-3/4 2xl:w-1/2 ">
                        {user_header_link_list}
                    </div>
                })
            }
        }}
        </TransitionUnpack>
    }
}
