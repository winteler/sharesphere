use std::str::FromStr;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use strum_macros::{Display, EnumString, IntoStaticStr};

use crate::app::GlobalState;
use crate::errors::AppError;
use crate::unpack::ArcTransitionUnpack;
#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::ssr::check_user, auth::ssr::reload_user, user::ssr::SqlUser};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserForumRole {
    pub role_id: i64,
    pub user_id: i64,
    pub username: String,
    pub forum_id: i64,
    pub forum_name: String,
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

    use crate::errors::AppError;
    use crate::user::{ssr::SqlUser, User};

    use super::*;

    pub async fn get_user_forum_role(
        user_id: i64,
        forum_name: &str,
        db_pool: &PgPool,
    ) -> Result<UserForumRole, AppError> {
        let user_forum_role = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles \
            WHERE user_id = $1 AND \
                  forum_name = $2",
            user_id,
            forum_name,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(user_forum_role)
    }

    pub async fn get_forum_role_vec(
        forum_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<UserForumRole>, AppError> {
        let forum_role_vec = sqlx::query_as!(
            UserForumRole,
            "SELECT * FROM user_forum_roles
            WHERE
                forum_name = $1 AND
                permission_level != 'None'",
            forum_name,
        )
            .fetch_all(db_pool)
            .await?;

        Ok(forum_role_vec)
    }

    pub async fn set_user_forum_role(
        user_id: i64,
        forum_name: &str,
        permission_level: PermissionLevel,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<(UserForumRole, Option<i64>), AppError> {
        if permission_level == PermissionLevel::Lead {
            set_forum_leader(user_id, forum_name, grantor, db_pool).await
        } else {
            grantor.check_can_set_user_forum_role(permission_level, user_id, forum_name, db_pool).await?;
            let user_forum_role = insert_user_forum_role(
                user_id,
                forum_name,
                permission_level,
                grantor,
                db_pool,
            ).await?;
            Ok((user_forum_role, None))
        }
    }
    async fn set_forum_leader(
        user_id: i64,
        forum_name: &str,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<(UserForumRole, Option<i64>), AppError> {
        match grantor.check_is_forum_leader(forum_name).is_ok() {
            true => {
                let manage_level_str: &str = PermissionLevel::Manage.into();
                sqlx::query_as!(
                    UserForumRole,
                    "UPDATE user_forum_roles
                    SET
                        permission_level = $1,
                        timestamp = CURRENT_TIMESTAMP
                    WHERE user_id = $2 AND
                          forum_name = $3
                    RETURNING *",
                    manage_level_str,
                    grantor.user_id,
                    forum_name,
                )
                    .fetch_one(db_pool)
                    .await?;
                let user_forum_role = insert_user_forum_role(
                    user_id,
                    forum_name,
                    PermissionLevel::Lead,
                    grantor,
                    db_pool,
                ).await?;
                Ok((user_forum_role, Some(grantor.user_id)))
            },
            false => {
                let user_forum_role = insert_user_forum_role(
                    user_id,
                    forum_name,
                    PermissionLevel::Lead,
                    grantor,
                    db_pool,
                ).await?;
                Ok((user_forum_role, None))
            },
        }
    }

    async fn insert_user_forum_role(
        user_id: i64,
        forum_name: &str,
        permission_level: PermissionLevel,
        grantor: &User,
        db_pool: &PgPool,
    ) -> Result<UserForumRole, AppError> {
        if user_id == grantor.user_id && grantor.check_is_forum_leader(forum_name).is_ok() {
            return Err(AppError::InternalServerError(String::from("Forum leader cannot lower his permissions, must designate another leader.")))
        }
        let permission_level_str: &str = permission_level.into();
        let user_forum_role = sqlx::query_as!(
                UserForumRole,
                "INSERT INTO user_forum_roles (user_id, username, forum_id, forum_name, permission_level, grantor_id)
                VALUES ($1,
                    (SELECT username from users where user_id = $1),
                    (SELECT forum_id FROM forums WHERE forum_name = $2),
                    $2, $3, $4)
                ON CONFLICT (user_id, forum_id) DO UPDATE
                SET permission_level = EXCLUDED.permission_level,
                    timestamp = CURRENT_TIMESTAMP
                RETURNING *",
                user_id,
                forum_name,
                permission_level_str,
                grantor.user_id,
            )
            .fetch_one(db_pool)
            .await?;
        Ok(user_forum_role)
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
pub async fn get_forum_role_vec(forum_name: String) -> Result<Vec<UserForumRole>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;

    let role_vec = ssr::get_forum_role_vec(
        &forum_name,
        &db_pool,
    ).await?;

    Ok(role_vec)
}

#[server]
pub async fn set_user_forum_role(
    username: String,
    forum_name: String,
    permission_level: PermissionLevel,
) -> Result<UserForumRole, ServerFnError<AppError>> {
    let user = check_user().await?;
    let db_pool = get_db_pool()?;

    let assigned_user = SqlUser::get_by_username(&username, &db_pool).await?;

    let (forum_role, _) = ssr::set_user_forum_role(
        assigned_user.user_id,
        &forum_name,
        permission_level,
        &user,
        &db_pool,
    ).await?;

    reload_user(forum_role.user_id)?;

    Ok(forum_role)
}

/// Component to show children when the user has at least the input permission level
#[component]
pub fn AuthorizedShow<C: IntoView + 'static>(
    #[prop(into)]
    forum_name: Signal<String>,
    permission_level: PermissionLevel,
    children: TypedChildrenFn<C>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let children = StoredValue::new(children.into_inner());
    view! {
        <ArcTransitionUnpack resource=state.user let:user>
            <Show when=move || match &*user {
                Some(user) => user.check_permissions(&forum_name.read(), permission_level).is_ok(),
                None => false,
            }>
            {
                children.with_value(|children| children())
            }
            </Show>
        </ArcTransitionUnpack>
    }.into_any()
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