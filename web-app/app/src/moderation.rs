use leptos::html;
use leptos::prelude::*;
use leptos_use::use_textarea_autosize;
use serde::{Deserialize, Serialize};

use crate::app::GlobalState;
use crate::comment::Comment;
use crate::content::{Content, ContentBody};
use crate::editor::{FormTextEditor, TextareaData};
use crate::errors::AppError;
use crate::icons::{HammerIcon, MagnifierIcon};
use crate::post::Post;
use crate::role::{AuthorizedShow, PermissionLevel};
use crate::rule::get_rule_by_id;
use crate::rule::Rule;
use crate::sphere::SphereState;
use crate::unpack::{ActionError, SuspenseUnpack, TransitionUnpack};
use crate::widget::{ModalDialog, ModalFormButtons};
#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::{check_user, reload_user},
    comment::ssr::{get_comment_by_id, get_comment_sphere},
    post::ssr::get_post_by_id,
    rule::ssr::load_rule_by_id
};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModerationInfo {
    pub rule: Rule,
    pub content: Content,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    
    use crate::comment::Comment;
    use crate::errors::AppError;
    use crate::post::Post;
    use crate::role::{AdminRole, PermissionLevel};
    use crate::sphere_management::{ssr::is_user_sphere_moderator, UserBan};
    use crate::user::User;
    
    pub async fn moderate_post(
        post_id: i64,
        rule_id: i64,
        moderator_message: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Post, AppError> {
        let post = if user.check_admin_role(AdminRole::Moderator).is_ok() {
            sqlx::query_as::<_, Post>(
                "UPDATE posts SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    post_id = $5
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(post_id)
                .fetch_one(db_pool)
                .await?
        } else {
            sqlx::query_as::<_, Post>(
                "UPDATE posts p SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    p.post_id = $5 AND
                    EXISTS (
                        SELECT * FROM user_sphere_roles r
                        WHERE
                            r.sphere_id = p.sphere_id AND
                            r.user_id = $3
                    )
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(post_id)
                .fetch_one(db_pool)
                .await?
        };

        Ok(post)
    }

    pub async fn moderate_comment(
        comment_id: i64,
        rule_id: i64,
        moderator_message: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let comment = if user.check_admin_role(AdminRole::Moderator).is_ok() {
            sqlx::query_as::<_, Comment>(
                "UPDATE comments SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    comment_id = $5
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(comment_id)
                .fetch_one(db_pool)
                .await?
        } else {
            // check if the user has at least the moderate permission for this sphere
            sqlx::query_as::<_, Comment>(
                "UPDATE comments c SET
                    moderator_message = $1,
                    infringed_rule_id = $2,
                    infringed_rule_title = (SELECT title FROM rules WHERE rule_id = $2),
                    edit_timestamp = CURRENT_TIMESTAMP,
                    moderator_id = $3,
                    moderator_name = $4
                WHERE
                    c.comment_id = $5 AND
                    EXISTS (
                        SELECT * FROM user_sphere_roles r
                        JOIN posts p ON p.sphere_id = r.sphere_id
                        WHERE
                            p.post_id = c.post_id AND
                            r.user_id = $3
                    )
                RETURNING *",
            )
                .bind(moderator_message)
                .bind(rule_id)
                .bind(user.user_id)
                .bind(user.username.clone())
                .bind(comment_id)
                .fetch_one(db_pool)
                .await?
        };

        Ok(comment)
    }

    pub async fn ban_user_from_sphere(
        user_id: i64,
        sphere_name: &String,
        post_id: i64,
        comment_id: Option<i64>,
        rule_id: i64,
        user: &User,
        ban_duration_days: Option<usize>,
        db_pool: &PgPool,
    ) -> Result<Option<UserBan>, AppError> {
        if user.check_permissions(&sphere_name, PermissionLevel::Moderate).is_ok() && user.user_id != user_id && !is_user_sphere_moderator(user_id, sphere_name, &db_pool).await? {
            let user_ban = match ban_duration_days {
                Some(0) => None,
                Some(ban_duration) => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "INSERT INTO user_bans (user_id, username, sphere_id, sphere_name, post_id, comment_id, infringed_rule_id, moderator_id, until_timestamp)
                         VALUES (
                            $1,
                            (SELECT username FROM users WHERE user_id = $1),
                            (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                            $2, $3, $4, $5, $6, CURRENT_TIMESTAMP + $7 * interval '1 day'
                        ) RETURNING *",
                        user_id,
                        sphere_name,
                        post_id,
                        comment_id,
                        rule_id,
                        user.user_id,
                        ban_duration as f64,
                    )
                        .fetch_one(db_pool)
                        .await?)
                }
                None => {
                    Some(sqlx::query_as!(
                        UserBan,
                        "INSERT INTO user_bans (user_id, username, sphere_id, sphere_name, post_id, comment_id, infringed_rule_id, moderator_id)
                         VALUES (
                            $1,
                            (SELECT username FROM users WHERE user_id = $1),
                            (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                            $2, $3, $4, $5, $6
                        ) RETURNING *",
                        user_id,
                        sphere_name,
                        post_id,
                        comment_id,
                        rule_id,
                        user.user_id,
                    )
                        .fetch_one(db_pool)
                        .await?)
                }
            };
            Ok(user_ban)
        } else {
            Err(AppError::InternalServerError(format!("Error while trying to ban user {user_id}. Insufficient permissions or user is a moderator of the sphere.")))
        }
    }
}

#[server]
pub async fn get_moderation_info(
    post_id: i64,
    comment_id: Option<i64>,
) -> Result<ModerationInfo, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let (rule_id, content) = match comment_id {
        Some(comment_id) => {
            let comment = get_comment_by_id(comment_id, &db_pool).await?;
            (comment.infringed_rule_id, Content::Comment(comment))
        },
        None => {
            let post = get_post_by_id(post_id, &db_pool).await?;
            (post.infringed_rule_id, Content::Post(post))
        },
    };
    let rule = match rule_id {
        Some(rule_id) => load_rule_by_id(rule_id, &db_pool).await,
        None => Err(AppError::InternalServerError(String::from("Content is not moderated, cannot find moderation info.")))
    }?;

    Ok(ModerationInfo {
        rule,
        content,
    })
}

/// Function to moderate a post and optionally ban its author
///
/// The ban is performed for the sphere of the given post and the duration is given by `ban_num_days`.
/// If `ban_num_days == None`, the duration of the ban is permanent.
#[server]
pub async fn moderate_post(
    post_id: i64,
    rule_id: i64,
    moderator_message: String,
    ban_duration_days: Option<usize>,
) -> Result<Post, ServerFnError<AppError>> {
    log::debug!("Moderate post {post_id}, ban duration = {ban_duration_days:?}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let post = ssr::moderate_post(
        post_id,
        rule_id,
        moderator_message.as_str(),
        &user,
        &db_pool
    ).await?;

    ssr::ban_user_from_sphere(
        post.creator_id,
        &post.sphere_name,
        post.post_id,
        None,
        rule_id,
        &user,
        ban_duration_days,
        &db_pool,
    ).await?;

    reload_user(post.creator_id)?;

    Ok(post)
}

/// Function to moderate a comment and optionally ban its author
///
/// The ban is performed for the sphere of the given comment and the duration is given by `ban_num_days`.
/// If `ban_num_days == None`, the duration of the ban is permanent.
#[server]
pub async fn moderate_comment(
    comment_id: i64,
    rule_id: i64,
    moderator_message: String,
    ban_duration_days: Option<usize>,
) -> Result<Comment, ServerFnError<AppError>> {
    log::trace!("Moderate comment {comment_id}");
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let comment = ssr::moderate_comment(
        comment_id,
        rule_id,
        moderator_message.as_str(),
        &user,
        &db_pool
    ).await?;

    let sphere = get_comment_sphere(comment_id, &db_pool).await?;

    ssr::ban_user_from_sphere(
        comment.creator_id,
        &sphere.sphere_name,
        comment.post_id,
        Some(comment.comment_id),
        rule_id,
        &user,
        ban_duration_days,
        &db_pool
    ).await?;

    reload_user(comment.creator_id)?;

    Ok(comment)
}

/// Displays the body of a moderated post or comment
#[component]
pub fn ModeratedBody(
    infringed_rule_title: String,
    moderator_message: String,
) -> impl IntoView {
    view! {
        <div class="flex items-stretch w-fit">
            <div class="flex justify-center items-center p-2 rounded-l bg-base-content/20">
                <HammerIcon/>
            </div>
            <div class="p-2 rounded-r bg-base-300 whitespace-pre align-middle">
                <div class="flex flex-col gap-1">
                    <div>{moderator_message}</div>
                    <div>{format!("Infringed rule: {infringed_rule_title}")}</div>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Component to moderate a post
#[component]
pub fn ModerateButton(show_dialog: RwSignal<bool>) -> impl IntoView {
    let edit_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    view! {
        <button
            class=edit_button_class
            aria-expanded=move || show_dialog.get().to_string()
            aria-haspopup="dialog"
            on:click=move |_| show_dialog.set(true)
        >
            <HammerIcon/>
        </button>
    }.into_any()
}

/// Component to access a post's moderation dialog
#[component]
pub fn ModeratePostButton(post_id: i64) -> impl IntoView {
    let show_dialog = RwSignal::new(false);
    let sphere_name = expect_context::<SphereState>().sphere_name;
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Moderate>
            <div>
                <ModerateButton show_dialog/>
                <ModeratePostDialog
                    post_id
                    show_dialog
                />
            </div>
        </AuthorizedShow>
    }.into_any()
}

/// Component to access a comment's moderation dialog
#[component]
pub fn ModerateCommentButton(comment_id: i64, comment: RwSignal<Comment>) -> impl IntoView {
    let show_dialog = RwSignal::new(false);
    let sphere_name = expect_context::<SphereState>().sphere_name;
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Moderate>
            <div>
                <ModerateButton show_dialog/>
                <ModerateCommentDialog
                    comment_id
                    comment
                    show_dialog
                />
            </div>
        </AuthorizedShow>
    }.into_any()
}

/// Dialog to moderate a post
#[component]
pub fn ModeratePostDialog(
    post_id: i64,
    show_dialog: RwSignal<bool>
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();

    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_autosize = use_textarea_autosize(textarea_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref,
    };
    let is_text_empty = Signal::derive(move || body_data.content.read().is_empty());

    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Moderate a post"</div>
                <ActionForm action=sphere_state.moderate_post_action>
                    <div class="flex flex-col gap-3 w-full">
                        <input
                            type="text"
                            name="post_id"
                            class="hidden"
                            value=post_id
                        />
                        <RuleSelect name="rule_id"/>
                        <FormTextEditor
                            name="moderator_message"
                            placeholder="Message"
                            data=body_data
                        />
                        <BanMenu/>
                        <ModalFormButtons
                            disable_publish=is_text_empty
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
                <ActionError action=sphere_state.moderate_post_action.into()/>
            </div>
        </ModalDialog>
    }.into_any()
}

/// Dialog to moderate a comment
#[component]
pub fn ModerateCommentDialog(
    comment_id: i64,
    comment: RwSignal<Comment>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let comment_autosize = use_textarea_autosize(textarea_ref);
    let comment_data = TextareaData{
        content: comment_autosize.content,
        set_content: comment_autosize.set_content,
        textarea_ref,
    };
    let is_text_empty = Signal::derive(move || comment_data.content.read().is_empty());

    let moderate_comment_action = ServerAction::<ModerateComment>::new();

    let moderate_result = moderate_comment_action.value();

    Effect::new(move |_| {
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
            <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Moderate a comment"</div>
                <ActionForm action=moderate_comment_action>
                    <div class="flex flex-col gap-3 w-full">
                        <input
                            type="text"
                            name="comment_id"
                            class="hidden"
                            value=comment_id
                        />
                        <RuleSelect name="rule_id"/>
                        <FormTextEditor
                            name="moderator_message"
                            placeholder="Message"
                            data=comment_data
                        />
                        <BanMenu/>
                        <ModalFormButtons
                            disable_publish=is_text_empty
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
                <ActionError action=moderate_comment_action.into()/>
            </div>
        </ModalDialog>
    }.into_any()
}

/// Dialog to select infringed rule
#[component]
pub fn RuleSelect(
    name: &'static str,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    view! {
        <div class="flex items-center justify-between w-full">
            <span class="text-xl font-semibold">"Infringed rule:"</span>
            <select
                class="select"
                name=name
            >
                <TransitionUnpack resource=sphere_state.sphere_rules_resource let:rules_vec>
                {
                    rules_vec.iter().map(|rule| {
                        view! {
                            <option value=rule.rule_id>
                                {rule.title.clone()}
                            </option>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </select>
        </div>
    }.into_any()
}

/// Dialog to input number of banned days
#[component]
pub fn BanMenu() -> impl IntoView {
    let ban_value = RwSignal::new(0);
    let is_permanent_ban = RwSignal::new(false);
    let sphere_name = expect_context::<SphereState>().sphere_name;
    view! {
        <input
            type="number"
            name="ban_duration_days"
            class="hidden"
            value=ban_value
            disabled=is_permanent_ban
        />
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Ban>
            <div class="flex items-center justify-between w-full">
                <span class="text-xl font-semibold">"Ban duration (days):"</span>
                <select
                    class="select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        if let Ok(num_days_banned) = value.parse::<i32>() {
                            ban_value.set(num_days_banned);
                            is_permanent_ban.set(false);
                        } else {
                            ban_value.set(0);
                            is_permanent_ban.set(true);
                        }
                    }
                >
                    <option>"0"</option>
                    <option>"1"</option>
                    <option>"7"</option>
                    <option>"30"</option>
                    <option>"180"</option>
                    <option>"365"</option>
                    <option value="">"Permanent"</option>
                </select>
            </div>
        </AuthorizedShow>
    }.into_any()
}

/// Displays the body of a moderated post or comment
#[component]
pub fn ModerationInfoButton(
    #[prop(into)]
    content: Signal<Content>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();
    let show_button = move || {
        let (is_moderated, creator_id) = match &*content.read() {
            Content::Post(post) => (post.infringed_rule_id.is_some(), post.creator_id),
            Content::Comment(comment) => (comment.infringed_rule_id.is_some(), comment.creator_id),
        };
        let is_author = match &(*state.user.read()) {
            Some(Ok(Some(user))) => user.user_id == creator_id,
            _ => false
        };
        let is_moderator = *sphere_state.permission_level.read() >= PermissionLevel::Moderate;
        is_moderated && (is_author || is_moderator)
    };
    let show_dialog = RwSignal::new(false);
    let button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };
    
    view! {
        <Suspense>
            <Show when=show_button>
                <button
                    class=button_class
                    on:click=move |_| show_dialog.update(|value| *value = !*value)
                >
                    <MagnifierIcon/>
                </button>
                <ModalDialog
                    class="w-full max-w-xl"
                    show_dialog
                >
                    <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
                        <ContentModerationInfo content=content/>
                        <button
                            type="button"
                            class="p-1 h-full rounded-xs bg-error hover:bg-error/75 active:scale-95 transition duration-250"
                            on:click=move |_| show_dialog.set(false)
                        >
                            "Close"
                        </button>
                    </div>
                </ModalDialog>
            </Show>
        </Suspense>
    }.into_any()
}

/// Component to display a button opening a modal dialog with a ban's details
#[component]
pub fn ContentModerationInfo(
    #[prop(into)]
    content: Signal<Content>,
) -> impl IntoView {
    let mod_info_resource = Resource::new(
        move || content.get(),
        move |content| async move {
            let rule_id = match &content {
                Content::Post(post) => post.infringed_rule_id,
                Content::Comment(comment) => comment.infringed_rule_id
            };
            match rule_id {
                Some(rule_id) => {
                    let rule = get_rule_by_id(rule_id).await?;
                    Ok(ModerationInfo {
                        content,
                        rule,
                    })
                },
                None => Err(ServerFnError::WrappedServerError(AppError::new("Content is not moderated."))),
            }
        }
    );

    view! {
        <SuspenseUnpack resource=mod_info_resource let:moderation_info>
            <ModerationInfoDialog moderation_info/>
        </SuspenseUnpack>
    }
}

/// Component to display the details of a moderation instance
#[component]
pub fn ModerationInfoDialog<'a>(
    moderation_info: &'a ModerationInfo,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <h1 class="text-center font-bold text-2xl">"Ban details"</h1>
            {
                match &moderation_info.content {
                    Content::Post(post) => view! {
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-2xl pl-6">"Content"</h1>
                            <div>{post.title.clone()}</div>
                            <ContentBody
                                body=post.body.clone()
                                is_markdown=post.markdown_body.is_some()
                            />
                        </div>
                        <div class="flex flex-col gap-1 p-2 border-b">
                            <h1 class="font-bold text-2xl pl-6">"Moderator message"</h1>
                            <div>{post.moderator_message.clone()}</div>
                        </div>
                    }.into_any(),
                    Content::Comment(comment) => {
                        view! {
                            <div class="flex flex-col gap-1 p-2 border-b">
                                <div class="font-bold text-2xl pl-6">"Content"</div>
                                <ContentBody
                                    body=comment.body.clone()
                                    is_markdown=comment.markdown_body.is_some()
                                />
                            </div>
                            <div class="flex flex-col gap-1 p-2 border-b">
                                <div class="font-bold text-2xl pl-6">"Moderator message"</div>
                                <div>{comment.moderator_message.clone()}</div>
                            </div>
                        }.into_any()
                    }
                }
            }
            <div class="flex flex-col gap-1 p-2">
                <h1 class="font-bold text-2xl pl-6">"Infringed rule"</h1>
                <div class="text-lg font-semibold">{moderation_info.rule.title.clone()}</div>
                <div>{moderation_info.rule.description.clone()}</div>
            </div>
        </div>
    }
}