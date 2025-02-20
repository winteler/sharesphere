use std::collections::BTreeSet;
use leptos::html;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::comment::CommentWithContext;
use crate::errors::AppError;
use crate::icons::MagnifierIcon;
use crate::post::PostWithSphereInfo;
use crate::sphere::{SphereHeader};
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

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::collections::BTreeSet;
    use sqlx::PgPool;
    use crate::comment::CommentWithContext;
    use crate::errors::AppError;
    use crate::post::PostWithSphereInfo;
    use crate::post::ssr::PostJoinCategory;
    use crate::sphere::SphereHeader;

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

    pub async fn get_matching_sphere_header_vec(
        sphere_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_header_vec = sqlx::query_as!(
            SphereHeader,
            "SELECT sphere_name, icon_url, is_nsfw
            FROM spheres
            WHERE sphere_name LIKE $1
            ORDER BY sphere_name LIMIT $2",
            format!("{sphere_prefix}%"),
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
}

#[server]
pub async fn get_matching_username_set(
    username_prefix: String,
) -> Result<BTreeSet<String>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let username_set = ssr::get_matching_username_set(&username_prefix, USER_FETCH_LIMIT, &db_pool).await?;
    Ok(username_set)
}

#[server]
pub async fn get_matching_sphere_header_vec(
    sphere_prefix: String,
) -> Result<Vec<SphereHeader>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_matching_sphere_header_vec(
        &sphere_prefix,
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

/// Button to open the search dialog
#[component]
pub fn SearchButton() -> impl IntoView
{
    let show_dialog = RwSignal::new(false);
    let modal_ref = NodeRef::<html::Div>::new();
    let _ = on_click_outside(modal_ref, move |_| show_dialog.set(false));
    view! {
        <button
            class="btn btn-ghost btn-circle"
            on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
            attr:aria-expanded=move || show_dialog.get().to_string()
            attr:aria-haspopup="dialog"
        >
            <MagnifierIcon/>
        </button>
        <ModalDialog show_dialog modal_ref>
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
        <EnumSignalTabs
            enum_signal=search_type
            enum_iter=SearchType::iter()
        />
    }
}

#[component]
pub fn SearchSphere() -> impl IntoView
{
    view! {
        <div>"Sphere"</div>
    }
}

#[component]
pub fn SearchPost() -> impl IntoView
{
    view! {
        <div>"Post"</div>
    }
}

#[component]
pub fn SearchComment() -> impl IntoView
{
    view! {
        <div>"Comment"</div>
    }
}

#[component]
pub fn SearchUser() -> impl IntoView
{
    view! {
        <div>"User"</div>
    }
}

/// Form for the search dialog
#[component]
pub fn SearchForm() -> impl IntoView
{

}

/// Component to display the result of the search
#[component]
pub fn SearchResult() -> impl IntoView
{

}