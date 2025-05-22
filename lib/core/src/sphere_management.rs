use std::env;
use leptos::prelude::*;
use server_fn::codec::{MultipartData, MultipartFormData};
use sharesphere_auth::user::UserBan;
use sharesphere_utils::errors::AppError;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::{ssr::check_user, ssr::reload_user},
        session::ssr::get_db_pool,
    },
};

pub const LEPTOS_SITE_ROOT_ENV: &str = "LEPTOS_SITE_ROOT";
pub const ICON_FOLDER: &str = "icons/";
pub const BANNER_FOLDER: &str = "banners/";

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::path::Path;
    use http::StatusCode;
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;
    use server_fn::codec::MultipartData;
    use sqlx::types::Uuid;
    use sqlx::PgPool;
    use tokio::fs::{rename, File};
    use tokio::io::AsyncWriteExt;

    use sharesphere_utils::constants::IMAGE_TYPE;
    use sharesphere_utils::errors::AppError;
    use sharesphere_utils::widget::{IMAGE_FILE_PARAM, SPHERE_NAME_PARAM};

    use sharesphere_auth::role::{AdminRole, PermissionLevel};
    use sharesphere_auth::user::{User, UserBan};

    pub const MAX_IMAGE_MB_SIZE: usize = 5; // 5 MB
    pub const MAX_IMAGE_SIZE: usize = MAX_IMAGE_MB_SIZE * 1024 * 1024; // 5 MB in bytes
    pub const MISSING_SPHERE_STR: &str = "Missing sphere name.";
    pub const MISSING_BANNER_FILE_STR: &str = "Missing banner file.";
    pub const INCORRECT_BANNER_FILE_TYPE_STR: &str = "Banner file must be an image.";
    pub const BANNER_FILE_INFER_ERROR_STR: &str = "Could not infer file extension.";
    pub async fn get_sphere_ban_vec(
        sphere_name: &str,
        username_prefix: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserBan>, AppError> {
        let user_ban_vec = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans
            WHERE sphere_name = $1 AND
                  username like $2
            ORDER BY until_timestamp DESC",
            sphere_name,
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

        match &user_ban.sphere_name {
            Some(sphere_name) => grantor.check_permissions(sphere_name, PermissionLevel::Ban),
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

    /// Extracts and stores a sphere associated image from `data` and returns the sphere name and file name for the image.
    ///
    /// The image will be stored locally on the server with the following path: <store_path><image_category><file_name>.
    /// Returns an error if the sphere name or file cannot be found, if the file does not contain a valid image file or
    /// if directories in the path <store_path><image_category> do not exist.
    pub async fn store_sphere_image(
        store_path: &str,
        image_category: &str,
        data: MultipartData,
        user: &User,
    ) -> Result<(String, Option<String>), AppError> {
        // `.into_inner()` returns the inner `multer` stream
        // it is `None` if we call this on the client, but always `Some(_)` on the server, so is safe to
        // unwrap
        let mut data = data.into_inner().unwrap();
        let mut sphere_name = Err(AppError::new(MISSING_SPHERE_STR));
        let mut file_field = Err(AppError::new(MISSING_BANNER_FILE_STR));

        while let Ok(Some(field)) = data.next_field().await {
            let name = field.name().unwrap_or_default().to_string();
            if name == SPHERE_NAME_PARAM {
                sphere_name = Ok(field.text().await.map_err(|e| AppError::new(e.to_string()))?);
            } else if name == IMAGE_FILE_PARAM {
                file_field = Ok(field);
            }
        }

        let sphere_name = sphere_name?;
        let mut file_field = file_field?;

        user.check_permissions(&sphere_name, PermissionLevel::Manage)?;

        if file_field.file_name().unwrap_or_default().is_empty() {
            return Ok((sphere_name, None))
        }

        let directory = Path::new(store_path).join(image_category);
        tokio::fs::create_dir_all(&directory).await?;
        let temp_file_path = directory.join(format!("image_{}", Uuid::new_v4()));

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

        let file_name = format!("{}.{}", sphere_name, file_extension);
        let image_path = directory.join(&file_name);

        // TODO delete previous file? Here or somewhere else?
        rename(&temp_file_path, &image_path).await?;
        Ok((sphere_name, Some(file_name)))
    }

    pub async fn set_sphere_icon_url(
        sphere_name: &str,
        icon_url: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;
        sqlx::query!(
            "UPDATE spheres
             SET icon_url = $1,
                 timestamp = CURRENT_TIMESTAMP
             WHERE sphere_name = $2",
            icon_url,
            sphere_name,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn set_sphere_banner_url(
        sphere_name: &str,
        banner_url: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;
        sqlx::query!(
            "UPDATE spheres
             SET banner_url = $1,
                 timestamp = CURRENT_TIMESTAMP
             WHERE sphere_name = $2",
            banner_url,
            sphere_name,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }
}

#[server]
pub async fn get_sphere_ban_vec(
    sphere_name: String,
    username_prefix: String,
) -> Result<Vec<UserBan>, AppError> {
    let db_pool = get_db_pool()?;
    let ban_vec = ssr::get_sphere_ban_vec(&sphere_name, &username_prefix, &db_pool).await?;
    Ok(ban_vec)
}

#[server]
pub async fn remove_user_ban(
    ban_id: i64
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;
    let deleted_user_ban = ssr::remove_user_ban(ban_id, &user, &db_pool).await?;
    reload_user(deleted_user_ban.user_id)?;
    Ok(())
}

#[server(input = MultipartFormData)]
pub async fn set_sphere_icon(
    data: MultipartData,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (sphere_name, icon_file_name) = ssr::store_sphere_image(&env::var(LEPTOS_SITE_ROOT_ENV)?, ICON_FOLDER, data, &user).await?;
    let icon_url = icon_file_name.map(|icon_file_name| format!("/{ICON_FOLDER}{icon_file_name}"));
    ssr::set_sphere_icon_url(&sphere_name.clone(), icon_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}

#[server(input = MultipartFormData)]
pub async fn set_sphere_banner(
    data: MultipartData,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let (sphere_name, banner_file_name) = ssr::store_sphere_image(&env::var(LEPTOS_SITE_ROOT_ENV)?, BANNER_FOLDER, data, &user).await?;
    let banner_url = banner_file_name.map(|banner_file_name| format!("/{BANNER_FOLDER}{banner_file_name}"));
    ssr::set_sphere_banner_url(&sphere_name.clone(), banner_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}