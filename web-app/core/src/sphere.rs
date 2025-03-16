use leptos::prelude::{Memo, Resource, RwSignal, ServerAction, Signal};
use serde::{Deserialize, Serialize};
use server_fn::ServerFnError;
use auth::role::{PermissionLevel, UserSphereRole};
use utils::colors::Color;
use utils::errors::AppError;
use crate::satellite::Satellite;

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
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SphereCategory {
    pub category_id: i64,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub category_name: String,
    pub category_color: Color,
    pub description: String,
    pub is_active: bool,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone)]
pub struct SphereState {
    pub sphere_name: Memo<String>,
    pub category_id_filter: RwSignal<Option<i64>>,
    pub permission_level: Signal<PermissionLevel>,
    pub sphere_resource: Resource<leptos::error::Result<Sphere, ServerFnError<AppError>>>,
    pub satellite_vec_resource: Resource<leptos::error::Result<Vec<Satellite>, ServerFnError<AppError>>>,
    pub sphere_categories_resource: Resource<leptos::error::Result<Vec<SphereCategory>, ServerFnError<AppError>>>,
    pub sphere_roles_resource: Resource<leptos::error::Result<Vec<UserSphereRole>, ServerFnError<AppError>>>,
    pub sphere_rules_resource: Resource<leptos::error::Result<Vec<Rule>, ServerFnError<AppError>>>,
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