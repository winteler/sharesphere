use chrono::SecondsFormat;
use leptos::html;
use leptos::prelude::*;
use leptos::web_sys::FormData;
use leptos_router::components::Outlet;
use leptos_use::{signal_debounced, use_textarea_autosize};
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use std::collections::BTreeSet;
use std::sync::Arc;
use strum::IntoEnumIterator;

use crate::app::{GlobalState, LoginWindow};
use crate::editor::{FormTextEditor, TextareaData};
use crate::errors::{AppError, ErrorDisplay};
use crate::forum::{Forum, ForumState};
use crate::forum_category::ForumCategoriesDialog;
use crate::icons::{DeleteIcon, MagnifierIcon, SaveIcon};
use crate::moderation::{get_moderation_info, ModerationInfoDialog};
use crate::role::{AuthorizedShow, PermissionLevel, SetUserForumRole};
use crate::rule::ForumRulesPanel;
use crate::unpack::{ArcSuspenseUnpack, ArcTransitionUnpack, SuspenseUnpack};
use crate::user::get_matching_username_set;
use crate::widget::{EnumDropdown, ForumImageForm, ModalDialog};
#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::{check_user, reload_user},
};

pub const MANAGE_FORUM_ROUTE: &str = "/manage";
pub const NONE_STR: &str = "None";
pub const DAY_STR: &str = "day";
pub const DAYS_STR: &str = "days";
pub const PERMANENT_STR: &str = "Permanent";
pub const ASSET_FOLDER: &str = "./public/";
pub const ICON_FOLDER: &str = "icons/";
pub const BANNER_FOLDER: &str = "banners/";
pub const MISSING_FORUM_STR: &str = "Missing forum name.";
pub const MISSING_BANNER_FILE_STR: &str = "Missing banner file.";
pub const INCORRECT_BANNER_FILE_TYPE_STR: &str = "Banner file must be an image.";
pub const BANNER_FILE_INFER_ERROR_STR: &str = "Could not infer file extension.";

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct UserBan {
    pub ban_id: i64,
    pub user_id: i64,
    pub username: String,
    pub forum_id: Option<i64>,
    pub forum_name: Option<String>,
    pub post_id: i64,
    pub comment_id: Option<i64>,
    pub infringed_rule_id: i64,
    pub moderator_id: i64,
    pub until_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use http::StatusCode;
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;
    use server_fn::codec::MultipartData;
    use sqlx::types::Uuid;
    use sqlx::PgPool;
    use tokio::fs::{rename, File};
    use tokio::io::AsyncWriteExt;

    use crate::constants::IMAGE_TYPE;
    use crate::errors::AppError;
    use crate::forum_management::{UserBan, BANNER_FILE_INFER_ERROR_STR, INCORRECT_BANNER_FILE_TYPE_STR, MISSING_BANNER_FILE_STR, MISSING_FORUM_STR};
    use crate::role::{AdminRole, PermissionLevel};
    use crate::user::User;
    use crate::widget::{FORUM_NAME_PARAM, IMAGE_FILE_PARAM};

    pub const MAX_IMAGE_MB_SIZE: usize = 5; // 5 MB
    pub const MAX_IMAGE_SIZE: usize = MAX_IMAGE_MB_SIZE * 1024 * 1024; // 5 MB in bytes

    pub async fn is_user_forum_moderator(
        user_id: i64,
        forum: &str,
        db_pool: &PgPool,
    ) -> Result<bool, AppError> {
        match User::get(user_id, db_pool).await {
            Some(user) => Ok(user.check_permissions(forum, PermissionLevel::Moderate).is_ok()),
            None => Err(AppError::InternalServerError(format!("Could not find user with id = {user_id}"))),
        }
    }

    pub async fn get_forum_ban_vec(
        forum_name: &str,
        username_prefix: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserBan>, AppError> {
        let user_ban_vec = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans
            WHERE forum_name = $1 AND
                  username like $2
            ORDER BY until_timestamp DESC",
            forum_name,
            format!("{username_prefix}%"),
        )
            .fetch_all(db_pool)
            .await?;

        Ok(user_ban_vec)
    }

    pub async fn remove_user_ban(
        ban_id: i64,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<UserBan, AppError> {
        let user_ban = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans WHERE ban_id = $1",
            ban_id
        )
            .fetch_one(db_pool)
            .await?;

        match &user_ban.forum_name {
            Some(forum_name) => grantor.check_permissions(forum_name, PermissionLevel::Ban),
            None => grantor.check_admin_role(AdminRole::Moderator),
        }?;

        sqlx::query!(
            "DELETE FROM user_bans WHERE ban_id = $1",
            ban_id
        )
            .execute(db_pool)
            .await?;

        Ok(user_ban)
    }

    /// Extracts and stores a forum associated image from `data` and returns the forum name and file name for the image.
    ///
    /// The image will be stored locally on the server with the following path: <store_path><image_category><file_name>.
    /// Returns an error if the forum name or file cannot be found, if the file does not contain a valid image file or
    /// if directories in the path <store_path><image_category> do not exist.
    pub async fn store_forum_image(
        store_path: &str,
        image_category: &str,
        data: MultipartData,
        user: &User,
    ) -> Result<(String, Option<String>), AppError> {
        // `.into_inner()` returns the inner `multer` stream
        // it is `None` if we call this on the client, but always `Some(_)` on the server, so is safe to
        // unwrap
        let mut data = data.into_inner().unwrap();
        let mut forum_name = Err(AppError::new(MISSING_FORUM_STR));
        let mut file_field = Err(AppError::new(MISSING_BANNER_FILE_STR));

        while let Ok(Some(field)) = data.next_field().await {
            let name = field.name().unwrap_or_default().to_string();
            if name == FORUM_NAME_PARAM {
                forum_name = Ok(field.text().await.map_err(|e| AppError::new(e.to_string()))?);
            } else if name == IMAGE_FILE_PARAM {
                file_field = Ok(field);
            }
        }

        let forum_name = forum_name?;
        let mut file_field = file_field?;

        user.check_permissions(&forum_name, PermissionLevel::Manage)?;

        if file_field.file_name().unwrap_or_default().is_empty() {
            return Ok((forum_name, None))
        }

        let temp_file_path = format!("/tmp/image_{}", Uuid::new_v4());

        let mut file = File::create(&temp_file_path).await?;
        let mut total_size = 0;
        while let Ok(Some(chunk)) = file_field.chunk().await {
            total_size += chunk.len();

            // Check if the total size exceeds the limit
            if total_size > MAX_IMAGE_SIZE {
                if let Some(response) = use_context::<ResponseOptions>() {
                    response.set_status(StatusCode::PAYLOAD_TOO_LARGE);
                }
                return Err(AppError::PayloadTooLarge(MAX_IMAGE_MB_SIZE));
            }
            file.write_all(&chunk).await?;
        }
        file.flush().await?;

        let file_extension = match infer::get_from_path(temp_file_path.clone()) {
            Ok(Some(file_type)) if file_type.mime_type().starts_with(IMAGE_TYPE) => Ok(file_type.extension()),
            Ok(Some(file_type)) => {
                log::info!("Invalid file type: {}, extension: {}", file_type.mime_type(), file_type.extension());
                Err(AppError::new(INCORRECT_BANNER_FILE_TYPE_STR))
            },
            Ok(None) => Err(AppError::new(BANNER_FILE_INFER_ERROR_STR)),
            Err(e) => Err(AppError::from(e)),
        }?;

        let image_path = format!("{}{}{}.{}", store_path, image_category, forum_name.clone(), file_extension);
        println!("Image path: {}", image_path);

        // TODO create folder?
        // TODO delete previous file? Here or somewhere else?
        rename(&temp_file_path, &image_path).await?;
        let file_name = format!("{}.{}", forum_name, file_extension);
        Ok((forum_name, Some(file_name)))
    }

    pub async fn set_forum_icon_url(
        forum_name: &str,
        icon_url: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(forum_name, PermissionLevel::Manage)?;
        sqlx::query!(
            "UPDATE forums
             SET icon_url = $1,
                 timestamp = CURRENT_TIMESTAMP
             WHERE forum_name = $2",
            icon_url,
            forum_name,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn set_forum_banner_url(
        forum_name: &str,
        banner_url: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(forum_name, PermissionLevel::Manage)?;
        sqlx::query!(
            "UPDATE forums
             SET banner_url = $1,
                 timestamp = CURRENT_TIMESTAMP
             WHERE forum_name = $2",
            banner_url,
            forum_name,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }
}

#[server]
pub async fn get_forum_ban_vec(
    forum_name: String,
    username_prefix: String,
) -> Result<Vec<UserBan>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let ban_vec = ssr::get_forum_ban_vec(&forum_name, &username_prefix, &db_pool).await?;
    Ok(ban_vec)
}

#[server]
pub async fn remove_user_ban(
    ban_id: i64
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;
    let deleted_user_ban = ssr::remove_user_ban(ban_id, &user, &db_pool).await?;
    reload_user(deleted_user_ban.user_id)?;
    Ok(())
}

#[server(input = MultipartFormData)]
pub async fn set_forum_icon(
    data: MultipartData,
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (forum_name, icon_file_name) = ssr::store_forum_image(ASSET_FOLDER, ICON_FOLDER, data, &user).await?;
    let icon_url = icon_file_name.map(|icon_file_name| format!("/{ICON_FOLDER}{icon_file_name}"));
    ssr::set_forum_icon_url(&forum_name.clone(), icon_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}

#[server(input = MultipartFormData)]
pub async fn set_forum_banner(
    data: MultipartData,
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (forum_name, banner_file_name) = ssr::store_forum_image(ASSET_FOLDER, BANNER_FOLDER, data, &user).await?;
    let banner_url = banner_file_name.map(|banner_file_name| format!("/{BANNER_FOLDER}{banner_file_name}"));
    ssr::set_forum_banner_url(&forum_name.clone(), banner_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}

/// Component to guard the forum cockpit
#[component]
pub fn ForumCockpitGuard() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let forum_name = expect_context::<ForumState>().forum_name;
    view! {
        <SuspenseUnpack resource=state.user let:user>
        {
            match user {
                Some(user) => {
                    match user.check_permissions(&forum_name.read_untracked(), PermissionLevel::Moderate) {
                        Ok(_) => view! { <Outlet/> }.into_any(),
                        Err(error) => view! { <ErrorDisplay error/> }.into_any(),
                    }
                },
                None => view! { <LoginWindow/> }.into_any(),
            }
        }
        </SuspenseUnpack>
    }.into_any()
}

/// Component to manage a forum
#[component]
pub fn ForumCockpit() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-5 overflow-y-auto w-full 2xl:w-1/2 mx-auto">
            <div class="text-2xl text-center">"Forum Cockpit"</div>
            <ForumDescriptionDialog/>
            <ForumIconDialog/>
            <ForumBannerDialog/>
            <ForumCategoriesDialog/>
            <ModeratorPanel/>
            <ForumRulesPanel/>
            <BanPanel/>
        </div>
    }.into_any()
}

/// Component to edit a forum's description
#[component]
pub fn ForumDescriptionDialog() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = expect_context::<ForumState>().forum_name;
    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
                <div class="text-xl text-center">"Forum description"</div>
                <ArcSuspenseUnpack resource=forum_state.forum_resource let:forum>
                    <ForumDescriptionForm forum=forum/>
                </ArcSuspenseUnpack>
            </div>
        </AuthorizedShow>
    }
}

/// Form to edit a forum's description
#[component]
pub fn ForumDescriptionForm(
    forum: Arc<Forum>,
) -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let description_autosize = use_textarea_autosize(textarea_ref);
    let description_data = TextareaData {
        content: description_autosize.content,
        set_content: description_autosize.set_content,
        textarea_ref
    };
    description_data.set_content.set(forum.description.clone());
    let disable_submit = move || description_data.content.read().is_empty();
    view! {
        <ActionForm
            action=forum_state.update_forum_desc_action
            attr:class="flex flex-col gap-1"
        >
            <input
                name="forum_name"
                class="hidden"
                value=forum_state.forum_name
            />
            <FormTextEditor
                name="description"
                placeholder="Description"
                data=description_data
            />
            <button
                type="submit"
                class="btn btn-secondary btn-sm p-1 self-end"
                disabled=disable_submit
            >
                <SaveIcon/>
            </button>
        </ActionForm>
    }
}

/// Component to edit a forum's icon
#[component]
pub fn ForumIconDialog() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;
    let set_icon_action = Action::new_local(|data: &FormData| {
        set_forum_icon(data.clone().into())
    });
    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
                <div class="text-xl text-center">"Forum icon"</div>
                <ForumImageForm
                    forum_name=forum_state.forum_name
                    action=set_icon_action
                    preview_class="max-h-12 max-w-full object-contain"
                />
            </div>
        </AuthorizedShow>
    }
}

/// Component to edit a forum's banner
#[component]
pub fn ForumBannerDialog() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;
    let set_banner_action = Action::new_local(|data: &FormData| {
        set_forum_banner(data.clone().into())
    });
    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
                <div class="text-xl text-center">"Forum banner"</div>
                <ForumImageForm
                    forum_name=forum_state.forum_name
                    action=set_banner_action
                />
            </div>
        </AuthorizedShow>
    }
}

/// Component to manage moderators
#[component]
pub fn ModeratorPanel() -> impl IntoView {
    let forum_state = expect_context::<ForumState>();
    let forum_name = forum_state.forum_name;
    let username_input = RwSignal::new(String::default());
    let select_ref = NodeRef::<html::Select>::new();

    let set_role_action = ServerAction::<SetUserForumRole>::new();

    view! {
        // TODO add overflow-y-auto max-h-full?
        <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Moderators"</div>
            <ArcTransitionUnpack resource=forum_state.forum_roles_resource let:forum_role_vec>
                <div class="flex flex-col gap-1">
                    <div class="flex gap-1 border-b border-base-content/20">
                        <div class="w-2/5 px-4 py-2 text-left font-bold">Username</div>
                        <div class="w-2/5 px-4 py-2 text-left font-bold">Role</div>
                    </div>
                    <For
                        each= move || (*forum_role_vec).clone().into_iter().enumerate()
                        key=|(_index, role)| (role.user_id, role.permission_level)
                        children=move |(_, role)| {
                            let username = StoredValue::new(role.username);
                            view! {
                                <div
                                    class="flex gap-1 py-1 rounded hover:bg-base-content/20 active:scale-95 transition duration-250"
                                    on:click=move |_| {
                                        username_input.set(username.get_value());
                                        match select_ref.get_untracked() {
                                            Some(select_ref) => select_ref.set_selected_index(role.permission_level as i32),
                                            None => log::error!("Form permission level select failed to load."),
                                        };
                                    }
                                >
                                    <div class="w-2/5 px-4 select-none">{username.get_value()}</div>
                                    <div class="w-2/5 px-4 select-none">{role.permission_level.to_string()}</div>
                                </div>
                            }
                        }
                    />
                </div>
            </ArcTransitionUnpack>
            <PermissionLevelForm
                forum_name
                username_input
                select_ref
                set_role_action
            />
        </div>
    }
}

/// Component to set permission levels for a forum
#[component]
pub fn PermissionLevelForm(
    forum_name: Memo<String>,
    username_input: RwSignal<String>,
    select_ref: NodeRef<html::Select>,
    set_role_action: ServerAction<SetUserForumRole>
) -> impl IntoView {
    let username_debounced: Signal<String> = signal_debounced(username_input, 250.0);
    let matching_user_resource = Resource::new(
        move || username_debounced.get(),
        move |username| async {
            if username.is_empty() {
                Ok(BTreeSet::<String>::default())
            } else {
                get_matching_username_set(username).await
            }
        },
    );
    let disable_submit = move || username_input.read().is_empty();

    view! {
        <AuthorizedShow forum_name permission_level=PermissionLevel::Manage>
            <ActionForm action=set_role_action>
                <input
                    name="forum_name"
                    class="hidden"
                    value=forum_name
                />
                <div class="flex gap-1 content-center">
                    <div class="dropdown dropdown-end w-2/5">
                        <input
                            tabindex="0"
                            type="text"
                            name="username"
                            placeholder="Username"
                            autocomplete="off"
                            class="input input-bordered input-primary w-full"
                            on:input=move |ev| {
                                username_input.set(event_target_value(&ev).to_lowercase());
                            }
                            prop:value=username_input
                        />
                        <Show when=move || !username_input.read().is_empty()>
                            <ArcTransitionUnpack resource=matching_user_resource let:username_set>
                                <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-200 rounded-box w-2/5">
                                    <For
                                        each= move || (*username_set).clone().into_iter().enumerate()
                                        key=|(_index, username)| username.clone()
                                        let:child
                                    >
                                        <li>
                                            <button type="button" value=child.1.clone() on:click=move |ev| username_input.set(event_target_value(&ev))>
                                                {child.1.clone()}
                                            </button>
                                        </li>
                                    </For>
                                </ul>
                            </ArcTransitionUnpack>
                        </Show>
                    </div>
                    <EnumDropdown
                        name="permission_level"
                        enum_iter=PermissionLevel::iter()
                        _select_ref=select_ref
                    />
                    <button
                        type="submit"
                        class="btn btn-secondary"
                        disabled=disable_submit
                    >
                        "Assign"
                    </button>
                </div>
            </ActionForm>
        </AuthorizedShow>
    }
}



/// Component to manage ban users
#[component]
pub fn BanPanel() -> impl IntoView {
    let forum_name = expect_context::<ForumState>().forum_name;
    let username_input = RwSignal::new(String::default());
    let username_debounced: Signal<String> = signal_debounced(username_input, 250.0);

    let unban_action = ServerAction::<RemoveUserBan>::new();
    let banned_users_resource = Resource::new(
        move || (username_debounced.get(), unban_action.version().get()),
        move |(username, _)| get_forum_ban_vec(forum_name.get_untracked(), username)
    );

    view! {
        // TODO add overflow-y-auto max-h-full?
        <div class="shrink-0 flex flex-col gap-1 content-center w-full bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Banned users"</div>
            <div class="flex flex-col gap-1">
                <div class="flex flex-col border-b border-base-content/20">
                    <div class="flex">
                        <input
                            class="input input-bordered input-primary px-6 w-2/5"
                            placeholder="Username"
                            value=username_input
                            on:input=move |ev| username_input.set(event_target_value(&ev))
                        />
                        <div class="w-2/5 px-6 py-2 text-left font-bold">Until</div>
                    </div>
                </div>
                <ArcTransitionUnpack resource=banned_users_resource let:banned_user_vec>
                    <For
                        each= move || (*banned_user_vec).clone().into_iter().enumerate()
                        key=|(_index, ban)| (ban.user_id, ban.until_timestamp)
                        let:child
                    >
                        <div class="flex">
                            <div class="w-2/5 px-6">{child.1.username}</div>
                            <div class="w-2/5 px-6">{
                                match child.1.until_timestamp {
                                    Some(until_timestamp) => until_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
                                    None => String::from("Permanent"),
                                }
                            }</div>
                            <div class="w-1/5 flex justify-end gap-1">
                                <BanInfoButton
                                    post_id=child.1.post_id
                                    comment_id=child.1.comment_id
                                />
                                <AuthorizedShow forum_name permission_level=PermissionLevel::Ban>
                                    <ActionForm action=unban_action>
                                        <input
                                            name="ban_id"
                                            class="hidden"
                                            value=child.1.ban_id
                                        />
                                        <button class="p-1 h-full rounded-sm bg-error hover:bg-error/75 active:scale-90 transition duration-250">
                                            <DeleteIcon/>
                                        </button>
                                    </ActionForm>
                                </AuthorizedShow>
                            </div>
                        </div>
                    </For>
                </ArcTransitionUnpack>
            </div>
        </div>
    }
}

/// Component to display a button opening a modal dialog with a ban's details
#[component]
pub fn BanInfoButton(
    post_id: i64,
    comment_id: Option<i64>,
) -> impl IntoView {
    let show_dialog = RwSignal::new(false);

    view! {
        <button
            class="p-1 h-full bg-secondary rounded-sm hover:bg-secondary/75 active:scale-90 transition duration-250"
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            <MagnifierIcon/>
        </button>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            {
                let ban_detail_resource = Resource::new(
                    move || (),
                    move |_| get_moderation_info(post_id, comment_id)
                );
                view! {
                    <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
                        <ArcSuspenseUnpack resource=ban_detail_resource let:moderation_info>
                            <ModerationInfoDialog moderation_info/>
                            <button
                                type="button"
                                class="p-1 h-full rounded-sm bg-error hover:bg-error/75 active:scale-95 transition duration-250"
                                on:click=move |_| show_dialog.set(false)
                            >
                                "Close"
                            </button>
                        </ArcSuspenseUnpack>
                    </div>
                }
            }
        </ModalDialog>
    }
}