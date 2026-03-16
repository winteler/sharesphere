use std::str::FromStr;

use leptos::prelude::*;
use leptos_fluent::{move_tr};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use strum_macros::{Display, EnumString, IntoStaticStr};

use sharesphere_utils::errors::AppError;
use sharesphere_utils::form::LabeledFormCheckbox;
use sharesphere_utils::unpack::SuspenseUnpack;

use crate::user::UserState;

#[cfg(feature = "ssr")]
use {
    sharesphere_utils::checks::{check_sphere_name, check_username},
    crate::{
        auth::ssr::{check_user, reload_user},
        user::ssr::SqlUser,
        session::ssr::get_db_pool,
    }
};

#[server]
pub async fn get_sphere_role_vec(sphere_name: String) -> Result<Vec<UserSphereRole>, AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;

    let role_vec = ssr::get_sphere_role_vec(
        &sphere_name,
        &db_pool,
    ).await?;

    Ok(role_vec)
}

#[server]
pub async fn set_user_sphere_role(
    username: String,
    sphere_name: String,
    permission_level: PermissionLevel,
) -> Result<UserSphereRole, AppError> {
    check_username(&username, false)?;
    check_sphere_name(&sphere_name)?;
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let assigned_user = SqlUser::get_by_username(&username, &db_pool).await?;

    let (sphere_role, prev_sphere_leader_id) = ssr::set_user_sphere_role(
        assigned_user.user_id,
        &sphere_name,
        permission_level,
        &user,
        &db_pool,
    ).await?;

    reload_user(sphere_role.user_id)?;

    if let Some(prev_leader_id) = prev_sphere_leader_id {
        // In case the sphere leader changed, also reload previous leader
        reload_user(prev_leader_id)?;
    };

    Ok(sphere_role)
}