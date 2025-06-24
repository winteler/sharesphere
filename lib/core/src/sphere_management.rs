use std::env;
use leptos::prelude::*;
use server_fn::codec::{MultipartData, MultipartFormData};
use sharesphere_auth::user::UserBan;
use sharesphere_utils::errors::AppError;

#[cfg(feature = "ssr")]
use {
    std::path::Path,
    sharesphere_auth::{
        auth::{ssr::check_user, ssr::reload_user},
        session::ssr::get_db_pool,
    },
    ssr::{
        MAX_BANNER_SIZE, MAX_ICON_SIZE, OBJECT_CONTAINER_URL_ENV, SphereImageType
    }
};

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::env;
    use http::StatusCode;
    use leptos::prelude::use_context;
    use leptos_axum::ResponseOptions;
    use object_store::aws::{AmazonS3, AmazonS3Builder};
    use object_store::{ObjectStore, PutPayload};
    use server_fn::codec::MultipartData;
    use sqlx::types::Uuid;
    use sqlx::PgPool;
    use url::Url;

    use sharesphere_utils::constants::IMAGE_TYPE;
    use sharesphere_utils::errors::AppError;
    use sharesphere_utils::widget::{IMAGE_FILE_PARAM, SPHERE_NAME_PARAM};

    use sharesphere_auth::role::{AdminRole, PermissionLevel};
    use sharesphere_auth::user::{User, UserBan};
    use crate::sphere::Sphere;
    use crate::sphere::ssr::get_sphere_by_name;

    pub const OBJECT_CONTAINER_URL_ENV: &str = "OBJECT_CONTAINER_URL";
    pub const ICON_BUCKET_ENV: &str = "ICON_BUCKET";
    pub const BANNER_BUCKET_ENV: &str = "BANNER_BUCKET";
    pub const MAX_ICON_SIZE: usize = 256 * 1024; // 0.25 MB in bytes
    pub const MAX_BANNER_SIZE: usize = 1024 * 1024; // 1 MB in bytes
    pub const MISSING_SPHERE_STR: &str = "Missing sphere name.";
    pub const MISSING_BANNER_FILE_STR: &str = "Missing banner file.";
    pub const INCORRECT_BANNER_FILE_TYPE_STR: &str = "Banner file must be an image.";
    pub const BANNER_FILE_INFER_ERROR_STR: &str = "Could not infer file extension.";

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]

    pub enum SphereImageType {
        ICON,
        BANNER,
    }

    impl SphereImageType {
        pub fn get_bucket_name(&self) -> Result<String, AppError> {
            let bucket_name = match self {
                SphereImageType::ICON => env::var(ICON_BUCKET_ENV),
                SphereImageType::BANNER => env::var(BANNER_BUCKET_ENV),
            }?;
            Ok(bucket_name)
        }

        pub fn get_sphere_image_url<'a>(&self, sphere: &'a Sphere) -> &'a Option<String> {
            match self {
                SphereImageType::ICON => &sphere.icon_url,
                SphereImageType::BANNER => &sphere.banner_url,
            }
        }
    }

    fn get_file_name_from_url(url_str: &str) -> Result<Option<String>, AppError> {
        let url = Url::parse(url_str)?;
        let file_name = match url.path_segments() {
            Some(s) => match s.last() {
                Some(ls) if ls.is_empty() || !ls.contains(".") => None,
                Some(ls) => Some(ls.to_string()),
                None => None,
            },
            None => None,
        };
        Ok(file_name)
    }

    pub async fn get_sphere_ban_vec(
        sphere_name: &str,
        username_prefix: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserBan>, AppError> {
        let user_ban_vec = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans
            WHERE sphere_name = $1 AND
                  username like $2 AND
                  delete_timestamp IS NULL
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
            "UPDATE user_bans SET delete_timestamp = CURRENT_TIMESTAMP WHERE ban_id = $1",
            ban_id
        )
            .execute(db_pool)
            .await?;

        Ok(user_ban)
    }

    pub(super) fn get_object_store(image_type: SphereImageType) -> Result<AmazonS3, AppError> {
        AmazonS3Builder::from_env()
            .with_bucket_name(image_type.get_bucket_name()?.clone())
            .build()
            .map_err(|e| AppError::new(format!("Error while building object store: {e}")))
    }

    /// Gets the current image url for the given `sphere_name` and tries to delete it
    pub async fn delete_sphere_image<T: ObjectStore>(
        sphere_name: &str,
        image_type: SphereImageType,
        object_store: &T,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_permissions(&sphere_name, PermissionLevel::Manage)?;
        let sphere = get_sphere_by_name(&sphere_name, db_pool).await?;
        if let Some(current_image_url) = image_type.get_sphere_image_url(&sphere) {
            if let Ok(Some(current_image_name)) = get_file_name_from_url(current_image_url) {
                let object_path = object_store::path::Path::from(current_image_name);
                if let Err(e) = object_store.delete(&object_path).await {
                    log::error!("Error while deleting current image: {e}");
                };
            } else {
                log::warn!("Could not parse file name for current image path: {}", current_image_url);
            }
        } else {
            log::info!("No image found for {sphere_name}");
        }
        Ok(())
    }

    /// Extracts and stores a sphere associated image from `data` and returns the sphere name and file name for the image.
    ///
    /// The image will be stored locally on the server with the following path: <store_path><image_category><file_name>.
    /// Returns an error if the sphere name or file cannot be found, if the file does not contain a valid image file or
    /// if directories in the path <store_path><image_category> do not exist.
    pub async fn store_sphere_image<T: ObjectStore>(
        data: MultipartData,
        max_image_size: usize,
        object_store: &T,
        user: &User,
    ) -> Result<(String, Option<String>), AppError> {
        // `.into_inner()` returns the inner `multer` stream
        // it is `None` if we call this on the client, but always `Some(_)` on the server, so is safe to unwrap
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

        let image_identifier = Uuid::new_v4();

        let mut input_file_buffer = Vec::<u8>::new();
        let mut total_size = 0;
        while let Ok(Some(chunk)) = file_field.chunk().await {
            total_size += chunk.len();

            // Check if the total size exceeds the limit
            if total_size > max_image_size {
                if let Some(response) = use_context::<ResponseOptions>() {
                    response.set_status(StatusCode::PAYLOAD_TOO_LARGE);
                }
                return Err(AppError::PayloadTooLarge(max_image_size));
            }
            input_file_buffer.append(chunk.to_vec().as_mut());
        }

        let file_extension = match infer::get(&input_file_buffer) {
            Some(file_type) if file_type.mime_type().starts_with(IMAGE_TYPE) => Ok(file_type.extension()),
            Some(file_type) => {
                log::info!("Invalid file type: {}, extension: {}", file_type.mime_type(), file_type.extension());
                Err(AppError::new(INCORRECT_BANNER_FILE_TYPE_STR))
            },
            None => Err(AppError::new(BANNER_FILE_INFER_ERROR_STR)),
        }?;

        // TODO compress image if needed?

        let file_name = format!("{}_{}.{}", sphere_name, image_identifier, file_extension);

        object_store.put(
            &object_store::path::Path::from(file_name.clone()),
            PutPayload::from_bytes(input_file_buffer.into())
        ).await.map_err(|e| AppError::new(format!("Error while uploading to object store: {e}")))?;

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

    #[cfg(test)]
    mod tests {
        use sealed_test::prelude::*;
        use crate::sphere::Sphere;
        use crate::sphere_management::ssr::{get_file_name_from_url, SphereImageType, BANNER_BUCKET_ENV, ICON_BUCKET_ENV};

        #[sealed_test]
        fn test_sphere_image_type_get_bucket_name() {
            unsafe {
                std::env::set_var(ICON_BUCKET_ENV, "a");
                std::env::remove_var(BANNER_BUCKET_ENV);
            }
            let icon = SphereImageType::ICON;
            let banner = SphereImageType::BANNER;
            assert_eq!(icon.get_bucket_name(), Ok(String::from("a")));
            assert!(banner.get_bucket_name().is_err());
        }

        #[test]
        fn test_sphere_image_type_get_sphere_image_url() {
            let icon = SphereImageType::ICON;
            let banner = SphereImageType::BANNER;

            let sphere = Sphere {
                sphere_id: 0,
                sphere_name: "a".to_string(),
                normalized_sphere_name: "a".to_string(),
                description: "b".to_string(),
                is_nsfw: false,
                is_banned: false,
                icon_url: Some("icon.png".to_string()),
                banner_url: Some("banner.jpg".to_string()),
                num_members: 0,
                creator_id: 0,
                create_timestamp: Default::default(),
                timestamp: Default::default(),
            };

            let sphere2 = Sphere {
                sphere_id: 1,
                sphere_name: "1".to_string(),
                normalized_sphere_name: "1".to_string(),
                description: "2".to_string(),
                is_nsfw: false,
                is_banned: false,
                icon_url: None,
                banner_url: None,
                num_members: 0,
                creator_id: 0,
                create_timestamp: Default::default(),
                timestamp: Default::default(),
            };

            assert_eq!(*icon.get_sphere_image_url(&sphere), Some(String::from("icon.png")));
            assert_eq!(*banner.get_sphere_image_url(&sphere), Some(String::from("banner.jpg")));
            assert_eq!(*icon.get_sphere_image_url(&sphere2), None);
            assert_eq!(*banner.get_sphere_image_url(&sphere2), None);
        }

        #[test]
        fn test_get_file_name_from_url() {
            let expected_file_name = String::from("test_image.jpg");
            let file_url = format!("https://storage.com/image/{expected_file_name}");
            assert_eq!(
                get_file_name_from_url(&file_url),
                Ok(Some(expected_file_name))
            );
            let no_file_url = "https://storage.com/image/just/an/url";
            assert_eq!(
                get_file_name_from_url(&no_file_url),
                Ok(None)
            );
            let not_an_url = "This is just text";
            assert!(get_file_name_from_url(&not_an_url).is_err());
        }
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

    let image_type = SphereImageType::ICON;
    let object_container_url = env::var(OBJECT_CONTAINER_URL_ENV)?;
    let bucket_name = image_type.get_bucket_name()?;
    let object_store = ssr::get_object_store(image_type)?;
    let (sphere_name, file_name) = ssr::store_sphere_image(data, MAX_ICON_SIZE, &object_store, &user).await?;
    // Clear previous image if it exists
    if let Err(e) = ssr::delete_sphere_image(&sphere_name, image_type, &object_store, &user, &db_pool).await {
        log::warn!("Failed to delete Sphere icon: {:?}", e);
    }

    let icon_url = file_name.map(|file_name| {
        Path::new(&object_container_url)
            .join(bucket_name)
            .join(&file_name)
            .to_string_lossy()
            .to_string()
    });
    ssr::set_sphere_icon_url(&sphere_name.clone(), icon_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}

#[server(input = MultipartFormData)]
pub async fn set_sphere_banner(
    data: MultipartData,
) -> Result<(), AppError> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let image_type = SphereImageType::BANNER;
    let object_container_url = env::var(OBJECT_CONTAINER_URL_ENV)?;
    let bucket_name = image_type.get_bucket_name()?;
    let object_store = ssr::get_object_store(image_type)?;
    let (sphere_name, file_name) = ssr::store_sphere_image(data, MAX_BANNER_SIZE, &object_store, &user).await?;
    // Clear previous image if it exists
    if let Err(e) = ssr::delete_sphere_image(&sphere_name, image_type, &object_store, &user, &db_pool).await  {
        log::warn!("Failed to delete Sphere banner: {:?}", e);
    }
    let banner_url = file_name.map(|file_name| {
        Path::new(&object_container_url)
            .join(bucket_name)
            .join(&file_name)
            .to_string_lossy()
            .to_string()
    });
    ssr::set_sphere_banner_url(&sphere_name.clone(), banner_url.as_deref(), &user, &db_pool).await?;
    Ok(())
}