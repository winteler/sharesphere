use const_format::concatcp;
use leptos::{component, create_effect, create_rw_signal, create_server_action, IntoView, RwSignal, server, ServerFnError, Show, SignalGet, SignalSet, SignalWith, use_context, view};
use leptos_router::ActionForm;

#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::ssr::check_user};
use crate::app::UserState;
use crate::comment::Comment;
use crate::editor::FormTextEditor;
use crate::icons::HammerIcon;
use crate::widget::{ActionError, ModalDialog, ModalFormButtons};

pub const MANAGE_FORUM_SUFFIX: &str = "manage";
pub const MANAGE_FORUM_ROUTE: &str = concatcp!("/", MANAGE_FORUM_SUFFIX);

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::auth::User;
    use crate::comment::Comment;
    use crate::errors::AppError;

    pub async fn moderate_comment(
        comment_id: i64,
        moderated_body: &str,
        user: &User,
        db_pool: PgPool,
    ) -> Result<Comment, AppError> {
        let comment = sqlx::query_as!(
            Comment,
            "UPDATE comments SET
                moderated_body = $1,
                edit_timestamp = CURRENT_TIMESTAMP
            WHERE
                comment_id = $2 AND
                creator_id = $3
            RETURNING *",
            moderated_body,
            comment_id,
            user.user_id,
        )
            .fetch_one(&db_pool)
            .await?;

        Ok(comment)
    }
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
        db_pool.clone(),
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
pub fn ModerateButton(
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let user_state = use_context::<UserState>();
    view! {
        <Show when=move || match user_state {
            Some(user_state) => user_state.can_moderate.get(),
            None => false,
        }>
            <button
                class="btn btn-circle btn-sm btn-ghost"
                on:click=move |_| show_dialog.set(true)
            >
                <HammerIcon/>
            </button>
        </Show>
    }
}

/// Component to access a post's moderation dialog
#[component]
pub fn ModeratePostButton() -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <ModerateButton show_dialog/>
    }
}

/// Component to access a comment's moderation dialog
#[component]
pub fn ModerateCommentButton() -> impl IntoView {
    let show_dialog = create_rw_signal(false);
    view! {
        <ModerateButton show_dialog/>
    }
}

/// Dialog to moderate a post
#[component]
pub fn ModeratePostDialog(
    _show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
    }
}

/// Dialog to moderate a comment
#[component]
pub fn ModerateCommentDialog(
    comment_id: i64,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let moderate_text = create_rw_signal(String::new());
    let is_text_empty = move || moderate_text.with(|comment: &String| comment.is_empty());

    let moderate_comment_action = create_server_action::<ModerateComment>();

    let edit_comment_result = moderate_comment_action.value();
    let has_error = move || edit_comment_result.with(|val| matches!(val, Some(Err(_))));

    // TODO: add ban option

    create_effect(move |_| {
        if let Some(Ok(_comment)) = edit_comment_result.get() {
            // TODO: update comment signal, make one signal with whole comment
            show_dialog.set(false);
        }
    });
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-200 p-4 flex flex-col gap-2">
                <ActionForm action=moderate_comment_action>
                    <div class="flex flex-col gap-2 w-full">
                        <input
                            type="text"
                            name="comment_id"
                            class="hidden"
                            value=comment_id
                        />
                        <FormTextEditor
                            name="moderate_body"
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

