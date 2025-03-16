use leptos::prelude::{Memo, Resource, RwSignal, ServerAction, Signal};
use leptos::server;
use serde::{Deserialize, Serialize};
use server_fn::ServerFnError;
use sharesphere_auth::role::{PermissionLevel, SetUserSphereRole, UserSphereRole};
use sharesphere_utils::errors::AppError;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::check_user,
        session::ssr::get_db_pool,
    },
};

use crate::moderation::ModeratePost;
use crate::rule::{AddRule, RemoveRule, Rule, UpdateRule};
use crate::satellite::{CreateSatellite, DisableSatellite, Satellite, UpdateSatellite};
use crate::sphere_category::{DeleteSphereCategory, SetSphereCategory, SphereCategory};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Sphere {
    pub sphere_id: i64,
    pub sphere_name: String,
    pub normalized_sphere_name: String,
    pub description: String,
    pub is_nsfw: bool,
    pub is_banned: bool,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub num_members: i32,
    pub creator_id: i64,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SphereHeader {
    pub sphere_name: String,
    pub icon_url: Option<String>,
    pub is_nsfw: bool,
}

#[derive(Copy, Clone)]
pub struct SphereState {
    pub sphere_name: Memo<String>,
    pub category_id_filter: RwSignal<Option<i64>>,
    pub permission_level: Signal<PermissionLevel>,
    pub sphere_resource: Resource<Result<Sphere, ServerFnError<AppError>>>,
    pub satellite_vec_resource: Resource<Result<Vec<Satellite>, ServerFnError<AppError>>>,
    pub sphere_categories_resource: Resource<Result<Vec<SphereCategory>, ServerFnError<AppError>>>,
    pub sphere_roles_resource: Resource<Result<Vec<UserSphereRole>, ServerFnError<AppError>>>,
    pub sphere_rules_resource: Resource<Result<Vec<Rule>, ServerFnError<AppError>>>,
    pub create_satellite_action: ServerAction<CreateSatellite>,
    pub update_satellite_action: ServerAction<UpdateSatellite>,
    pub disable_satellite_action: ServerAction<DisableSatellite>,
    pub moderate_post_action: ServerAction<ModeratePost>,
    pub update_sphere_desc_action: ServerAction<UpdateSphereDescription>,
    pub set_sphere_category_action: ServerAction<SetSphereCategory>,
    pub delete_sphere_category_action: ServerAction<DeleteSphereCategory>,
    pub set_sphere_role_action: ServerAction<SetUserSphereRole>,
    pub add_rule_action: ServerAction<AddRule>,
    pub update_rule_action: ServerAction<UpdateRule>,
    pub remove_rule_action: ServerAction<RemoveRule>,
}

impl From<&Sphere> for SphereHeader {
    fn from(sphere: &Sphere) -> Self {
        Self::new(sphere.sphere_name.clone(), sphere.icon_url.clone(), sphere.is_nsfw)
    }
}

impl SphereHeader {
    pub fn new(sphere_name: String, icon_url: Option<String>, is_nsfw: bool) -> Self {
        Self {
            sphere_name,
            icon_url,
            is_nsfw,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::PermissionLevel;
    use sharesphere_auth::user::User;
    use sharesphere_utils::errors::AppError;
    use crate::sphere::Sphere;

    pub async fn update_sphere_description(
        sphere_name: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Sphere, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;
        let sphere = sqlx::query_as::<_, Sphere>(
            "UPDATE spheres SET description = $1, timestamp = CURRENT_TIMESTAMP WHERE sphere_name = $2 RETURNING *"
        )
            .bind(description)
            .bind(sphere_name)
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }
}

#[server]
pub async fn update_sphere_description(
    sphere_name: String,
    description: String,
) -> Result<(), ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    ssr::update_sphere_description(&sphere_name, &description, &user, &db_pool).await?;

    Ok(())
}