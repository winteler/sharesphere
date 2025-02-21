use std::collections::BTreeSet;
use leptos::html;
use leptos::prelude::*;
use leptos_use::{on_click_outside, signal_debounced};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::comment::CommentWithContext;
use crate::errors::AppError;
use crate::form::LabeledSignalCheckbox;
use crate::icons::MagnifierIcon;
use crate::post::PostWithSphereInfo;
use crate::sphere::{SphereHeader, SphereLinkList};
use crate::unpack::TransitionUnpack;
use crate::widget::{EnumSignalTabs, ModalDialog, ToView};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    sphere::SPHERE_FETCH_LIMIT,
    user::USER_FETCH_LIMIT,
};

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SearchType {
    #[default]
    Sphere,
    Posts,
    Comments,
    User,
}

#[derive(Clone, Debug)]
struct SearchState {
    pub search_input: RwSignal<String>,
    pub search_input_debounced: Signal<String>,
    pub show_spoiler: RwSignal<bool>,
    pub show_nsfw: RwSignal<bool>,
}

impl ToView for SearchType {
    fn to_view(self) -> AnyView {
        match self {
            SearchType::Sphere => view! { <SearchSphere/> }.into_any(),
            SearchType::Posts => view! { <SearchPost/> }.into_any(),
            SearchType::Comments => view! { <SearchComment/> }.into_any(),
            SearchType::User => view! { <SearchUser/> }.into_any(),
        }
    }
}

impl Default for SearchState {
    fn default() -> Self {
        let search_input = RwSignal::new(String::new());
        SearchState {
            search_input,
            search_input_debounced: signal_debounced(search_input, 500.0),
            show_spoiler: RwSignal::new(false),
            show_nsfw: RwSignal::new(false),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::collections::BTreeSet;
    use sqlx::PgPool;
    use crate::comment::CommentWithContext;
    use crate::errors::AppError;
    use crate::post::PostWithSphereInfo;
    use crate::post::ssr::PostJoinCategory;
    use crate::sphere::SphereHeader;

    pub async fn get_matching_sphere_header_vec(
        sphere_prefix: &str,
        show_nsfw: bool,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_header_vec = sqlx::query_as!(
            SphereHeader,
            "SELECT sphere_name, icon_url, is_nsfw
            FROM spheres
            WHERE
                sphere_name LIKE $1 AND
                ($2 OR NOT is_nsfw)
            ORDER BY sphere_name LIMIT $3",
            format!("{sphere_prefix}%"),
            show_nsfw,
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_header_vec)
    }

    pub async fn search_posts(
        search_query: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let post_vec = sqlx::query_as::<_, PostJoinCategory>(
            "SELECT p.*, c.category_name, c.category_color, s.icon_url as sphere_icon_url FROM (
                SELECT *, ts_rank(document, plainto_tsquery('simple', $1)) AS rank
                FROM posts
                WHERE document @@ plainto_tsquery('simple', $1)
                ORDER BY rank DESC
            ) p
            JOIN spheres s on s.sphere_id = p.sphere_id
            LEFT JOIN sphere_categories c on c.category_id = p.category_id"
        )
            .bind(search_query)
            .fetch_all(db_pool)
            .await?;

        let post_vec = post_vec.into_iter().map(PostJoinCategory::into_post_with_sphere_info).collect();

        Ok(post_vec)
    }

    pub async fn search_comments(
        search_query: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<CommentWithContext>, AppError> {
        let comment_vec = sqlx::query_as::<_, CommentWithContext>(
            "SELECT c.*, s.sphere_name, s.icon_url, s.is_nsfw, p.satellite_id, p.title as post_title FROM (
                SELECT *, ts_rank(document, plainto_tsquery('simple', $1)) AS rank
                FROM comments
                WHERE document @@ plainto_tsquery('simple', $1)
                ORDER BY rank DESC
            ) c
            JOIN posts p ON p.post_id = c.post_id
            JOIN spheres s ON s.sphere_id = p.sphere_id"
        )
            .bind(search_query)
            .fetch_all(db_pool)
            .await?;

        Ok(comment_vec)
    }

    pub async fn get_matching_username_set(
        username_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<BTreeSet<String>, AppError> {
        let username_vec = sqlx::query!(
            "SELECT username FROM users WHERE username LIKE $1 ORDER BY username LIMIT $2",
            format!("{username_prefix}%"),
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        let mut username_set = BTreeSet::<String>::new();

        for row in username_vec {
            username_set.insert(row.username);
        }

        Ok(username_set)
    }
}

#[server]
pub async fn get_matching_sphere_header_vec(
    sphere_prefix: String,
    show_nsfw: bool,
) -> Result<Vec<SphereHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_matching_sphere_header_vec(
        &sphere_prefix,
        show_nsfw,
        SPHERE_FETCH_LIMIT,
        &db_pool
    ).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn search_posts(
    search_query: String,
) -> Result<Vec<PostWithSphereInfo>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let post_vec = ssr::search_posts(&search_query, &db_pool).await?;
    Ok(post_vec)
}

#[server]
pub async fn search_comments(
    search_query: String,
) -> Result<Vec<CommentWithContext>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let comment_vec = ssr::search_comments(&search_query, &db_pool).await?;
    Ok(comment_vec)
}

#[server]
pub async fn get_matching_username_set(
    username_prefix: String,
) -> Result<BTreeSet<String>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let username_set = ssr::get_matching_username_set(&username_prefix, USER_FETCH_LIMIT, &db_pool).await?;
    Ok(username_set)
}

/// Button to open the search dialog
#[component]
pub fn SearchButton() -> impl IntoView
{
    let show_dialog = RwSignal::new(false);
    let modal_ref = NodeRef::<html::Div>::new();
    let _ = on_click_outside(modal_ref, move |_| show_dialog.set(false));
    provide_context(SearchState::default());
    view! {
        <button
            class="btn btn-ghost btn-circle"
            on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
            attr:aria-expanded=move || show_dialog.get().to_string()
            attr:aria-haspopup="dialog"
        >
            <MagnifierIcon/>
        </button>
        <ModalDialog 
            show_dialog 
            modal_ref
            class="w-full max-w-xl"
        >
            <Search/>
        </ModalDialog>
    }
}

/// Component to search spheres, posts, comments and users
#[component]
pub fn Search() -> impl IntoView
{
    let search_type = RwSignal::new(SearchType::Sphere);
    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Search"</div>
            <EnumSignalTabs
                enum_signal=search_type
                enum_iter=SearchType::iter()
            />
        </div>
    }
}

#[component]
pub fn SearchSphere() -> impl IntoView
{
    let search_state = expect_context::<SearchState>();
    let search_sphere_resource = Resource::new(
        move || (search_state.search_input_debounced.get(), search_state.show_nsfw.get()),
        move |(search_input, show_nsfw)| async move {
            match search_input.is_empty() {
               true => Ok(Vec::new()),
               false => get_matching_sphere_header_vec(search_input, show_nsfw).await,
            }
        }
    );
    view! {
        <SearchForm
            show_spoiler_checkbox=false
            show_nsfw_checkbox=true
        />
        <TransitionUnpack resource=search_sphere_resource let:sphere_header_vec>
            <SphereLinkList sphere_header_vec=sphere_header_vec.clone()/>
        </TransitionUnpack>
    }
}

#[component]
pub fn SearchPost() -> impl IntoView
{
    view! {
        <SearchForm
            show_spoiler_checkbox=true
            show_nsfw_checkbox=true
        />
    }
}

#[component]
pub fn SearchComment() -> impl IntoView
{
    view! {
        <SearchForm
            show_spoiler_checkbox=false
            show_nsfw_checkbox=false
        />
    }
}

#[component]
pub fn SearchUser() -> impl IntoView
{
    view! {
        <SearchForm
            show_spoiler_checkbox=false
            show_nsfw_checkbox=true
        />
    }
}

/// Form for the search dialog
#[component]
pub fn SearchForm(
    show_spoiler_checkbox: bool,
    show_nsfw_checkbox: bool,
) -> impl IntoView {
    let search_state = expect_context::<SearchState>();
    let input_ref = NodeRef::<html::Input>::new();
    Effect::new(move || {
        if let Some(input_ref) = input_ref.get() {
            input_ref.focus().ok();
        } else {
            log::info!("Missing input ref");
        }
    });

    view! {
        <input
            type="text"
            placeholder="Search"
            class="input input-bordered input-primary h-input_m"
            value=search_state.search_input
            autofocus
            on:input=move |ev| search_state.search_input.set(event_target_value(&ev))
            node_ref=input_ref
        />
        { match show_spoiler_checkbox {
            true => Some(view! {
                <LabeledSignalCheckbox label="Spoiler" value=search_state.show_spoiler/>
            }),
            false => None,
        }}
         { match show_nsfw_checkbox {
            true => Some(view! {
                <LabeledSignalCheckbox label="NSFW" value=search_state.show_nsfw/>
            }),
            false => None,
        }}
    }
}