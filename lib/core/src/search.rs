use leptos::html;
use leptos::prelude::*;
use leptos_use::{signal_debounced, signal_throttled_with_options, ThrottleOptions};
use sharesphere_auth::user::UserHeader;
use sharesphere_utils::constants::{SCROLL_LOAD_THROTTLE_DELAY};
use sharesphere_utils::errors::AppError;
use sharesphere_utils::form::LabeledSignalCheckbox;
use sharesphere_utils::unpack::{handle_additional_load, handle_initial_load};
use crate::comment::{CommentWithContext};
use crate::post::{PostWithSphereInfo};
use crate::sphere::{InfiniteSphereLinkList, SphereHeader};

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::get_user,
        session::ssr::get_db_pool,
    },
    sharesphere_utils::checks::{check_sphere_name, check_string_length, check_username},
    sharesphere_utils::constants::{MAX_CONTENT_SEARCH_LENGTH},
    crate::post::POST_BATCH_SIZE,
    crate::comment::COMMENT_BATCH_SIZE,
};

#[derive(Clone, Debug)]
pub struct SearchState {
    pub search_input: RwSignal<String>,
    pub search_input_debounced: Signal<String>,
    pub show_spoiler: RwSignal<bool>,
}

impl Default for SearchState {
    fn default() -> Self {
        let search_input = RwSignal::new(String::new());
        SearchState {
            search_input,
            search_input_debounced: signal_debounced(search_input, 500.0),
            show_spoiler: RwSignal::new(false),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::cmp::min;
    use sqlx::PgPool;
    use sharesphere_auth::user::{UserHeader, USER_FETCH_LIMIT};
    use sharesphere_utils::errors::AppError;
    use crate::comment::CommentWithContext;
    use crate::post::PostWithSphereInfo;
    use crate::post::ssr::PostJoinCategory;
    use crate::sphere::{SphereHeader, SPHERE_FETCH_LIMIT};

    pub async fn get_matching_sphere_header_vec(
        sphere_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_header_vec = sqlx::query_as!(
            SphereHeader,
            "SELECT sphere_name, icon_url, is_nsfw
            FROM spheres
            WHERE normalized_sphere_name LIKE normalize_sphere_name($1)
            ORDER BY sphere_name LIMIT $2",
            format!("{sphere_prefix}%"),
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_header_vec)
    }

    pub async fn search_spheres(
        search_query: &str,
        show_nsfw: bool,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<SphereHeader>, AppError> {
        let sphere_vec = sqlx::query_as::<_, SphereHeader>(
            "WITH search AS (
                    SELECT *, 0.5 as rank
                    FROM spheres
                    WHERE
                        normalized_sphere_name LIKE format_for_search($1 || '%') AND
                        ($2 OR NOT is_nsfw) AND
                        NOT is_banned
                    UNION ALL
                    SELECT ws.*
                    FROM (
                        SELECT *, word_similarity(normalized_sphere_name, format_for_search($1)) as rank
                        FROM spheres
                        WHERE $2 OR NOT is_nsfw AND
                        NOT is_banned
                    ) ws
                    WHERE rank > 0.3
                    UNION ALL
                    SELECT ts.*
                    FROM (
                        SELECT *, ts_rank(sphere_document, plainto_tsquery('simple', $1)) as rank
                        FROM spheres
                        WHERE
                            sphere_document @@ plainto_tsquery('simple', $1) AND
                            ($2 OR NOT is_nsfw) AND
                            NOT is_banned
                    ) ts
                    WHERE rank > 0.01
                )
                SELECT ts.sphere_name, ts.icon_url, ts.is_nsfw
                FROM (
                    SELECT * FROM (
                        SELECT DISTINCT ON (sphere_name) * FROM search
                        ORDER BY sphere_name, rank DESC, num_members DESC
                    ) ts_distinct
                    ORDER BY rank DESC, num_members DESC
                ) ts
                LIMIT $3
                OFFSET $4"
        )
            .bind(search_query)
            .bind(show_nsfw)
            .bind(min(limit, SPHERE_FETCH_LIMIT as i64))
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_vec)
    }

    pub async fn search_posts(
        search_query: &str,
        sphere_name: Option<&str>,
        show_spoilers: bool,
        show_nsfw: bool,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<PostWithSphereInfo>, AppError> {
        let post_vec = sqlx::query_as::<_, PostJoinCategory>(
            "SELECT p.*, c.category_name, c.category_color, s.icon_url as sphere_icon_url FROM (
                SELECT *, ts_rank(post_document, plainto_tsquery('simple', $1)) AS rank
                FROM posts
                WHERE
                    post_document @@ plainto_tsquery('simple', $1) AND
                    ($2 IS NULL OR sphere_name = $2) AND
                    ($3 OR NOT is_spoiler) AND
                    ($4 OR NOT is_nsfw) AND
                    moderator_id IS NULL AND
                    delete_timestamp IS NULL
                ORDER BY rank DESC, score DESC
                LIMIT $5
                OFFSET $6
            ) p
            JOIN spheres s on s.sphere_id = p.sphere_id
            LEFT JOIN sphere_categories c on c.category_id = p.category_id"
        )
            .bind(search_query)
            .bind(sphere_name)
            .bind(show_spoilers)
            .bind(show_nsfw)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        let post_vec = post_vec.into_iter().map(PostJoinCategory::into_post_with_sphere_info).collect();

        Ok(post_vec)
    }

    pub async fn search_comments(
        search_query: &str,
        sphere_name: Option<&str>,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<CommentWithContext>, AppError> {
        let comment_vec = sqlx::query_as::<_, CommentWithContext>(
            "SELECT cp.*, s.sphere_name, s.icon_url, s.is_nsfw FROM (
                SELECT c.*, p.sphere_id, p.satellite_id, p.title as post_title, ts_rank(comment_document, plainto_tsquery('simple', $1)) AS rank
                FROM comments c
                JOIN posts p ON p.post_id = c.post_id
                WHERE
                    comment_document @@ plainto_tsquery('simple', $1) AND
                    c.moderator_id IS NULL AND
                    c.delete_timestamp IS NULL AND
                    ($2 IS NULL OR p.sphere_name = $2)
                ORDER BY rank DESC, score DESC
                LIMIT $3
                OFFSET $4
            ) cp
            JOIN spheres s ON s.sphere_id = cp.sphere_id"
        )
            .bind(search_query)
            .bind(sphere_name)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(comment_vec)
    }

    pub async fn get_matching_user_header_vec(
        username_prefix: &str,
        show_nsfw: bool,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<UserHeader>, AppError> {
        let user_header_vec = sqlx::query_as!(
            UserHeader,
            "SELECT username, is_nsfw
            FROM users
            WHERE
                username LIKE $1 AND
                ($2 OR NOT is_nsfw) AND
                delete_timestamp IS NULL
            ORDER BY username LIMIT $3",
            format!("{username_prefix}%"),
            show_nsfw,
            min(limit, USER_FETCH_LIMIT),
        )
            .fetch_all(db_pool)
            .await?;

        Ok(user_header_vec)
    }
}

#[server]
pub async fn get_matching_sphere_header_vec(
    sphere_prefix: String,
) -> Result<Vec<SphereHeader>, AppError> {
    check_sphere_name(&sphere_prefix)?;
    let db_pool = get_db_pool()?;
    let sphere_header_vec = ssr::get_matching_sphere_header_vec(
        &sphere_prefix,
        10,
        &db_pool
    ).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn search_spheres(
    search_query: String,
    load_count: usize,
    num_already_loaded: usize,
) -> Result<Vec<SphereHeader>, AppError> {
    check_sphere_name(&search_query)?;
    let db_pool = get_db_pool()?;
    let show_nsfw = get_user().await.unwrap_or(None).map(|user| user.show_nsfw).unwrap_or_default();
    let sphere_header_vec = ssr::search_spheres(&search_query, show_nsfw, load_count as i64, num_already_loaded as i64, &db_pool).await?;
    Ok(sphere_header_vec)
}

#[server]
pub async fn search_posts(
    search_query: String,
    sphere_name: Option<String>,
    show_spoilers: bool,
    num_already_loaded: usize,
) -> Result<Vec<PostWithSphereInfo>, AppError> {
    if let Some(sphere_name) = &sphere_name {
        check_sphere_name(sphere_name)?;
    }
    check_string_length(&search_query, "Search query", MAX_CONTENT_SEARCH_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let show_nsfw = get_user().await.unwrap_or(None).map(|user| user.show_nsfw).unwrap_or_default();
    let post_vec = ssr::search_posts(&search_query, sphere_name.as_deref(), show_spoilers, show_nsfw, POST_BATCH_SIZE, num_already_loaded as i64, &db_pool).await?;
    Ok(post_vec)
}

#[server]
pub async fn search_comments(
    search_query: String,
    sphere_name: Option<String>,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithContext>, AppError> {
    if let Some(sphere_name) = &sphere_name {
        check_sphere_name(sphere_name)?;
    }
    check_string_length(&search_query, "Search query", MAX_CONTENT_SEARCH_LENGTH, false)?;
    let db_pool = get_db_pool()?;
    let comment_vec = ssr::search_comments(&search_query, sphere_name.as_deref(), COMMENT_BATCH_SIZE, num_already_loaded as i64, &db_pool).await?;
    Ok(comment_vec)
}

#[server]
pub async fn get_matching_user_header_vec(
    username_prefix: String,
    show_nsfw: Option<bool>,
    load_count: usize,
) -> Result<Vec<UserHeader>, AppError> {
    check_username(username_prefix.as_str())?;
    let db_pool = get_db_pool()?;
    let show_nsfw = show_nsfw.unwrap_or_default() || get_user().await.unwrap_or(None).map(|user| user.show_nsfw).unwrap_or_default();
    let user_header_vec = ssr::get_matching_user_header_vec(&username_prefix, show_nsfw, load_count as i64, &db_pool).await?;
    Ok(user_header_vec)
}

#[component]
pub fn SearchSpheres(
    search_state: SearchState,
    #[prop(default = "gap-4 w-3/4 lg:w-1/2")]
    class: &'static str,
    #[prop(default = "w-full")]
    form_class: &'static str,
    #[prop(default = true)]
    autofocus: bool,
) -> impl IntoView
{
    let class = format!("flex flex-col self-center min-h-0 {class}");

    let sphere_header_vec = RwSignal::new(Vec::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();
    let num_fetch_sphere = 50;

    let _initial_sphere_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let search_input = search_state.search_input_debounced.get();
            let initial_load = match search_input.is_empty() {
                true => Ok(Vec::new()),
                false => search_spheres(search_input, num_fetch_sphere, 0).await,
            };
            handle_initial_load(initial_load, sphere_header_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let additional_load_count_throttled: Signal<i32> = signal_throttled_with_options(
        additional_load_count,
        SCROLL_LOAD_THROTTLE_DELAY,
        ThrottleOptions::default().leading(true).trailing(false)
    );

    let _additional_sphere_resource = LocalResource::new(
        move || async move {
            if additional_load_count_throttled.get() > 0 {
                is_loading.set(true);
                let sphere_count = sphere_header_vec.read_untracked().len();
                let search_input = search_state.search_input_debounced.get_untracked();
                let additional_load = search_spheres(search_input, num_fetch_sphere, sphere_count).await;
                handle_additional_load(additional_load, sphere_header_vec, load_error);
                is_loading.set(false);
            }
        }
    );
    view! {
        <div class=class>
            <SearchForm
                search_state
                show_spoiler_checkbox=false
                class=form_class
                autofocus
            />
            <div class="bg-base-200 rounded-sm min-h-0 max-h-full">
                <InfiniteSphereLinkList
                    sphere_header_vec
                    is_loading
                    load_error
                    additional_load_count
                    list_ref
                />
            </div>
        </div>
    }
}

/// Form for the search dialog
#[component]
pub fn SearchForm(
    search_state: SearchState,
    show_spoiler_checkbox: bool,
    #[prop(default = "w-3/4 lg:w-1/2")]
    class: &'static str,
    #[prop(default = true)]
    autofocus: bool,
) -> impl IntoView {
    let input_ref = NodeRef::<html::Input>::new();
    if autofocus {
        Effect::new(move || if let Some(input) = input_ref.get() {
            input.focus().ok();
        });
    }
    let class = format!("flex flex-col gap-2 self-center {class}");
    view! {
        <div class=class>
            <input
                type="text"
                placeholder="Search"
                class="input input-primary w-full"
                value=search_state.search_input
                autofocus=autofocus
                on:input=move |ev| search_state.search_input.set(event_target_value(&ev))
                node_ref=input_ref
            />
            { match show_spoiler_checkbox {
                true => Some(view! {
                    <LabeledSignalCheckbox label="Spoiler" value=search_state.show_spoiler class="pl-1"/>
                }),
                false => None,
            }}
        </div>
    }
}