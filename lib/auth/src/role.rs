use std::str::FromStr;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use strum_macros::{Display, EnumString, IntoStaticStr};

use sharesphere_utils::errors::AppError;
use sharesphere_utils::form::LabeledFormCheckbox;
use sharesphere_utils::unpack::SuspenseUnpack;

use crate::user::UserState;

#[cfg(feature = "ssr")]
use crate::{
    auth::ssr::{check_user, reload_user},
    user::ssr::SqlUser,
    session::ssr::get_db_pool,
};

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum AdminRole {
    None = 0,
    Moderator = 1,
    Admin = 2,
}

#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum PermissionLevel {
    None = 0,
    Moderate = 1,
    Ban = 2,
    Manage = 3,
    Lead = 4,
}

// TODO: add SCD2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserSphereRole {
    pub role_id: i64,
    pub user_id: i64,
    pub username: String,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub permission_level: PermissionLevel,
    pub grantor_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<String> for PermissionLevel {
    fn from(value: String) -> PermissionLevel {
        PermissionLevel::from_str(&value).unwrap_or(PermissionLevel::None)
    }
}

impl AdminRole {
    pub fn get_permission_level(self) -> PermissionLevel {
        match self {
            AdminRole::None => PermissionLevel::None,
            AdminRole::Moderator => PermissionLevel::Ban,
            AdminRole::Admin => PermissionLevel::Lead,
        }
    }
}

impl From<String> for AdminRole {
    fn from(value: String) -> AdminRole {
        AdminRole::from_str(&value).unwrap_or(AdminRole::None)
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use sharesphere_utils::errors::AppError;
    use crate::user::{ssr::SqlUser, User};

    use super::*;

    pub async fn get_user_sphere_role(
        user_id: i64,
        sphere_name: &str,
        db_pool: &PgPool,
    ) -> Result<UserSphereRole, AppError> {
        let user_sphere_role = sqlx::query_as!(
            UserSphereRole,
            "SELECT * FROM user_sphere_roles \
            WHERE user_id = $1 AND \
                  sphere_name = $2",
            user_id,
            sphere_name,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(user_sphere_role)
    }

    pub async fn is_user_sphere_moderator(
        user_id: i64,
        sphere: &str,
        db_pool: &PgPool,
    ) -> std::result::Result<bool, AppError> {
        match User::get(user_id, db_pool).await {
            Some(user) => Ok(user.check_permissions(sphere, PermissionLevel::Moderate).is_ok()),
            None => Err(AppError::InternalServerError(format!("Could not find user with id = {user_id}"))),
        }
    }

    pub async fn get_sphere_role_vec(
        sphere_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserSphereRole>, AppError> {
        let sphere_role_vec = sqlx::query_as!(
            UserSphereRole,
            "SELECT * FROM user_sphere_roles
            WHERE
                sphere_name = $1 AND
                permission_level != 'None'",
            sphere_name,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_role_vec)
    }

    pub async fn set_user_sphere_role(
        user_id: i64,
        sphere_name: &str,
        permission_level: PermissionLevel,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<(UserSphereRole, Option<i64>), AppError> {
        if permission_level == PermissionLevel::Lead {
            set_sphere_leader(user_id, sphere_name, grantor, db_pool).await
        } else {
            grantor.check_can_set_user_sphere_role(permission_level, user_id, sphere_name, db_pool).await?;
            let user_sphere_role = insert_user_sphere_role(
                user_id,
                sphere_name,
                permission_level,
                grantor,
                db_pool,
            ).await?;
            Ok((user_sphere_role, None))
        }
    }
    async fn set_sphere_leader(
        user_id: i64,
        sphere_name: &str,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<(UserSphereRole, Option<i64>), AppError> {
        match grantor.check_is_sphere_leader(sphere_name).is_ok() {
            true => {
                let manage_level_str: &str = PermissionLevel::Manage.into();
                sqlx::query_as!(
                    UserSphereRole,
                    "UPDATE user_sphere_roles
                    SET
                        permission_level = $1,
                        timestamp = CURRENT_TIMESTAMP
                    WHERE
                        user_id = $2 AND
                        sphere_name = $3
                    RETURNING *",
                    manage_level_str,
                    grantor.user_id,
                    sphere_name,
                )
                    .fetch_one(db_pool)
                    .await?;
                let user_sphere_role = insert_user_sphere_role(
                    user_id,
                    sphere_name,
                    PermissionLevel::Lead,
                    grantor,
                    db_pool,
                ).await?;
                Ok((user_sphere_role, Some(grantor.user_id)))
            },
            false => {
                let user_sphere_role = insert_user_sphere_role(
                    user_id,
                    sphere_name,
                    PermissionLevel::Lead,
                    grantor,
                    db_pool,
                ).await?;
                Ok((user_sphere_role, None))
            },
        }
    }

    async fn insert_user_sphere_role(
        user_id: i64,
        sphere_name: &str,
        permission_level: PermissionLevel,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<UserSphereRole, AppError> {
        if user_id == grantor.user_id && grantor.check_is_sphere_leader(sphere_name).is_ok() {
            return Err(AppError::InternalServerError(String::from("Sphere leader cannot lower his permissions, must designate another leader.")))
        }
        let permission_level_str: &str = permission_level.into();
        let user_sphere_role = sqlx::query_as!(
                UserSphereRole,
                "INSERT INTO user_sphere_roles (user_id, username, sphere_id, sphere_name, permission_level, grantor_id)
                VALUES ($1,
                    (SELECT username from users where user_id = $1),
                    (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                    $2, $3, $4)
                ON CONFLICT (user_id, sphere_id) DO UPDATE
                SET permission_level = EXCLUDED.permission_level,
                    timestamp = CURRENT_TIMESTAMP
                RETURNING *",
                user_id,
                sphere_name,
                permission_level_str,
                grantor.user_id,
            )
            .fetch_one(db_pool)
            .await?;
        Ok(user_sphere_role)
    }

    pub async fn set_user_admin_role(
        user_id: i64,
        admin_role: AdminRole,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<SqlUser, AppError> {
        grantor.check_admin_role(AdminRole::Admin)?;
        let admin_role_str: &str = admin_role.into();
        let sql_user = sqlx::query_as!(
            SqlUser,
            "UPDATE users
            SET
                admin_role = $1,
                timestamp = CURRENT_TIMESTAMP
            WHERE user_id = $2
            RETURNING *",
            admin_role_str,
            user_id,
        )
            .fetch_one(db_pool)
            .await?;
        Ok(sql_user)
    }
}

#[server]
pub async fn get_sphere_role_vec(sphere_name: String) -> Result<Vec<UserSphereRole>, ServerFnError<AppError>> {
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
) -> Result<UserSphereRole, ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let assigned_user = SqlUser::get_by_username(&username, &db_pool).await?;

    let (sphere_role, _) = ssr::set_user_sphere_role(
        assigned_user.user_id,
        &sphere_name,
        permission_level,
        &user,
        &db_pool,
    ).await?;

    reload_user(sphere_role.user_id)?;

    Ok(sphere_role)
}

/// Component to show children when the user has at least the input permission level
#[component]
pub fn AuthorizedShow<C: IntoView + 'static>(
    #[prop(into)]
    sphere_name: Signal<String>,
    permission_level: PermissionLevel,
    children: TypedChildrenFn<C>,
) -> impl IntoView {
    let user_state = expect_context::<UserState>();
    let children = StoredValue::new(children.into_inner());
    view! {
        <SuspenseUnpack resource=user_state.user let:user>
        {
            match user {
                Some(user) if user.check_permissions(&sphere_name.read(), permission_level).is_ok() => {
                    Some(children.with_value(|children| children()))
                },
                _ => None,
            }
        }
        </SuspenseUnpack>
    }.into_any()
}

#[component]
pub fn IsPinnedCheckbox(
    #[prop(into)]
    sphere_name: Signal<String>,
    #[prop(default = false)]
    value: bool,
) -> impl IntoView {
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Moderate>
            <LabeledFormCheckbox name="is_pinned" label="Pinned" value/>
        </AuthorizedShow>
    }
}

#[cfg(test)]
mod tests {
    use crate::role::{AdminRole, PermissionLevel};

    #[test]
    fn test_permission_level_from_string() {
        assert_eq!(PermissionLevel::from(String::from("None")), PermissionLevel::None);
        assert_eq!(PermissionLevel::from(String::from("Moderate")), PermissionLevel::Moderate);
        assert_eq!(PermissionLevel::from(String::from("Ban")), PermissionLevel::Ban);
        assert_eq!(PermissionLevel::from(String::from("Manage")), PermissionLevel::Manage);
        assert_eq!(PermissionLevel::from(String::from("Lead")), PermissionLevel::Lead);
        assert_eq!(PermissionLevel::from(String::from("invalid")), PermissionLevel::None);
    }

    #[test]
    fn test_admin_role_get_permission_level() {
        assert_eq!(AdminRole::None.get_permission_level(), PermissionLevel::None);
        assert_eq!(AdminRole::Moderator.get_permission_level(), PermissionLevel::Ban);
        assert_eq!(AdminRole::Admin.get_permission_level(), PermissionLevel::Lead);
    }

    #[test]
    fn test_admin_role_from_string() {
        assert_eq!(AdminRole::from(String::from("None")), AdminRole::None);
        assert_eq!(AdminRole::from(String::from("Moderator")), AdminRole::Moderator);
        assert_eq!(AdminRole::from(String::from("Admin")), AdminRole::Admin);
        assert_eq!(AdminRole::from(String::from("invalid")), AdminRole::None);
    }
}