use const_format::concatcp;
use leptos::{component, create_effect, create_rw_signal, create_server_action, expect_context, IntoView, RwSignal, server, ServerFnError, Show, SignalGet, SignalSet, SignalWith, use_context, view};
use leptos_router::ActionForm;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::ssr::check_user};
use crate::app::ModerateState;
use crate::comment::Comment;
use crate::editor::FormTextEditor;
use crate::errors::AppError;
use crate::icons::HammerIcon;
use crate::post::Post;
use crate::widget::{ActionError, EnumDropdown, ModalDialog, ModalFormButtons};

pub const MANAGE_FORUM_SUFFIX: &str = "manage";
pub const MANAGE_FORUM_ROUTE: &str = concatcp!("/", MANAGE_FORUM_SUFFIX);
pub const NONE_STR: &str = "None";
pub const ONE_DAY_STR: &str = "1 day";
pub const ONE_WEEK_STR: &str = "1 week";
pub const ONE_MONTH_STR: &str = "1 month";
pub const SIX_MONTH_STR: &str = "6 months";
pub const ONE_YEAR_STR: &str = "1 year";
pub const PERMANENT_STR: &str = "Permanent";

#[derive(Debug, PartialEq, EnumIter)]
pub enum BanDuration {
    None,
    Day,
    Week,
    Month,
    SixMonth,
    Year,
    Permanent,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct UserBan {
    pub ban_id: i64,
    pub user_id: i64,
    pub forum_id: Option<i64>,
    pub forum_name: Option<String>,
    pub until_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
}

impl std::str::FromStr for BanDuration {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            NONE_STR => Ok(BanDuration::None),
            ONE_DAY_STR => Ok(BanDuration::Day),
            ONE_WEEK_STR => Ok(BanDuration::Week),
            ONE_MONTH_STR => Ok(BanDuration::Month),
            SIX_MONTH_STR => Ok(BanDuration::SixMonth),
            ONE_YEAR_STR => Ok(BanDuration::Year),
            PERMANENT_STR => Ok(BanDuration::Permanent),
            _ => Err(AppError::InternalServerError(String::from("Failed to create BanDuration from &str"))),
        }
    }
}

impl From<BanDuration> for &'static str {
    fn from(value: BanDuration) -> Self {
        match value {
            BanDuration::None => NONE_STR,
            BanDuration::Day => ONE_DAY_STR,
            BanDuration::Week => ONE_WEEK_STR,
            BanDuration::Month => ONE_MONTH_STR,
            BanDuration::SixMonth => SIX_MONTH_STR,
            BanDuration::Year => ONE_YEAR_STR,
            BanDuration::Permanent => PERMANENT_STR,
        }
    }
}

impl From<&BanDuration> for &'static str {
    fn from(value: &BanDuration) -> Self {
        match value {
            BanDuration::None => NONE_STR,
            BanDuration::Day => ONE_DAY_STR,
            BanDuration::Week => ONE_WEEK_STR,
            BanDuration::Month => ONE_MONTH_STR,
            BanDuration::SixMonth => SIX_MONTH_STR,
            BanDuration::Year => ONE_YEAR_STR,
            BanDuration::Permanent => PERMANENT_STR,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::auth::User;
    use crate::comment::Comment;
    use crate::errors::AppError;
    use crate::post::Post;

    pub async fn moderate_post(
        post_id: i64,
        moderated_body: &str,
        user: &User,
        db_pool: PgPool,
    ) -> Result<Post, AppError> {

        let post = if user.is_global_moderator() {
            sqlx::query_as!(
                Post,
                "UPDATE posts SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    post_id = $4
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                post_id,
            )
                .fetch_one(&db_pool)
                .await?
        } else {
            sqlx::query_as!(
                Post,
                "UPDATE posts p SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    p.post_id = $4 AND
                    EXISTS (
                        SELECT * FROM user_forum_roles r
                        WHERE
                            r.forum_id = p.forum_id AND
                            r.user_id = $2
                    )
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                post_id,
            )
                .fetch_one(&db_pool)
                .await?
        };

        Ok(post)
    }

    pub async fn moderate_comment(
        comment_id: i64,
        moderated_body: &str,
        user: &User,
        db_pool: PgPool,
    ) -> Result<Comment, AppError> {
        let comment = if user.is_global_moderator() {
            sqlx::query_as!(
                Comment,
                "UPDATE comments SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    comment_id = $4
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                comment_id,
            )
                .fetch_one(&db_pool)
                .await?
        } else {
            sqlx::query_as!(
                Comment,
                "UPDATE comments c SET
                    moderated_body = $1,
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $2,
                    moderator_name = $3
                WHERE
                    c.comment_id = $4 AND
                    EXISTS (
                        SELECT * FROM user_forum_roles r
                        JOIN posts p ON p.forum_id = r.forum_id
                        WHERE
                            p.post_id = c.post_id AND
                            r.user_id = $2
                    )
                RETURNING *",
                moderated_body,
                user.user_id,
                user.username,
                comment_id,
            )
                .fetch_one(&db_pool)
                .await?
        };

        Ok(comment)
    }
}

#[server]
pub async fn moderate_post(
    post_id: i64,
    moderated_body: String,
    ban_duration: String,
) -> Result<Post, ServerFnError> {
    log::info!("Moderate post {post_id}, ban duration = {ban_duration}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let ban_duration_enum: BanDuration = ban_duration.parse::<BanDuration>()?;
    let ban_duration_str: &str = (&ban_duration_enum).into();
    log::info!("Ban duration: {ban_duration_enum:?}, str: {ban_duration_str}");

    let post = ssr::moderate_post(
        post_id,
        moderated_body.as_str(),
        &user,
        db_pool.clone()
    ).await?;

    Ok(post)
}

#[server]
pub async fn moderate_comment(
    comment_id: i64,
    moderated_body: String,
) -> Result<Comment, ServerFnError> {
    log::trace!("Moderate comment {comment_id}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let comment = ssr::moderate_comment(
        comment_id,
        moderated_body.as_str(),
        &user,
        db_pool.clone()
    ).await?;

    Ok(comment)
}

/// Component to manage a forum
#[component]
pub fn ForumCockpit() -> impl IntoView {
    view! {
        <div>
            "Forum Cockpit"
        </div>
    }
}

/// Component to moderate a post
#[component]
pub fn ModerateButton(show_dialog: RwSignal<bool>) -> impl IntoView {
    let moderate_state = use_context::<ModerateState>();
    let edit_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <Show when=move || match moderate_state {
            Some(user_state) => user_state.can_moderate.get(),
            None => false,
        }>
            <button
                class=edit_button_class
                aria-expanded=move || show_dialog.get().to_string()
                aria-haspopup="dialog"
                on:click=move |_| show_dialog.set(true)
            >
                <HammerIcon/>
            </button>
        </Show>
    }
}

/// Component to access a post's moderation dialog
#[component]
pub fn ModeratePostButton(post_id: i64) -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <div>
            <ModerateButton show_dialog/>
            <ModeratePostDialog
                post_id
                show_dialog
            />
        </div>
    }
}

/// Component to access a comment's moderation dialog
#[component]
pub fn ModerateCommentButton(comment_id: i64, comment: RwSignal<Comment>) -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <div>
            <ModerateButton show_dialog/>
            <ModerateCommentDialog
                comment_id
                comment
                show_dialog
            />
        </div>
    }
}

/// Dialog to moderate a post
#[component]
pub fn ModeratePostDialog(
    post_id: i64,
    show_dialog: RwSignal<bool>
) -> impl IntoView {
    let moderate_state = expect_context::<ModerateState>();

    let moderate_text = create_rw_signal(String::new());
    let is_text_empty = move || moderate_text.with(|moderate_text: &String| moderate_text.is_empty());

    let moderate_result = moderate_state.moderate_post_action.value();
    let has_error = move || moderate_result.with(|val| matches!(val, Some(Err(_))));

    // TODO: add ban option

    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Moderate a post"</div>
                <ActionForm action=moderate_state.moderate_post_action>
                    <div class="flex flex-col gap-3 w-full">
                        <input
                            type="text"
                            name="post_id"
                            class="hidden"
                            value=post_id
                        />
                        <FormTextEditor
                            name="moderated_body"
                            placeholder="Message"
                            content=moderate_text
                        />
                        <EnumDropdown name="ban_duration" enum_iter=BanDuration::iter()/>
                        <ModalFormButtons
                            disable_publish=is_text_empty
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
                <ActionError has_error/>
            </div>
        </ModalDialog>
    }
}

/// Dialog to moderate a comment
#[component]
pub fn ModerateCommentDialog(
    comment_id: i64,
    comment: RwSignal<Comment>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let moderate_text = create_rw_signal(String::new());
    let is_text_empty = move || moderate_text.with(|comment: &String| comment.is_empty());

    let moderate_comment_action = create_server_action::<ModerateComment>();

    let moderate_result = moderate_comment_action.value();
    let has_error = move || moderate_result.with(|val| matches!(val, Some(Err(_))));

    // TODO: add ban option

    create_effect(move |_| {
        if let Some(Ok(moderated_comment)) = moderate_result.get() {
            comment.set(moderated_comment);
            show_dialog.set(false);
        }
    });
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Moderate a comment"</div>
                <ActionForm action=moderate_comment_action>
                    <div class="flex flex-col gap-3 w-full">
                        <input
                            type="text"
                            name="comment_id"
                            class="hidden"
                            value=comment_id
                        />
                        <FormTextEditor
                            name="moderated_body"
                            placeholder="Message"
                            content=moderate_text
                        />
                        <ModalFormButtons
                            disable_publish=is_text_empty
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
                <ActionError has_error/>
            </div>
        </ModalDialog>
    }
}
