use std::cmp::max;
use std::collections::{BTreeSet, HashMap};

use leptos::{server, ServerFnError};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::app::ssr::get_db_pool;
use crate::errors::AppError;
use crate::role::{AdminRole, PermissionLevel};

pub const USER_FETCH_LIMIT: i64 = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum BanStatus {
    None,
    Until(chrono::DateTime<chrono::Utc>),
    Permanent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub oidc_id: String,
    pub username: String,
    pub email: String,
    pub admin_role: AdminRole,
    pub permission_by_forum_map: HashMap<String, PermissionLevel>,
    pub ban_status: BanStatus,
    pub ban_status_by_forum_map: HashMap<String, BanStatus>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_deleted: bool,
}

impl BanStatus {
    pub fn is_permanent(&self) -> bool {
        *self == BanStatus::Permanent
    }
    pub fn is_active(&self) -> bool {
        match self {
            BanStatus::Until(until_timestamp) => *until_timestamp > chrono::offset::Utc::now(),
            _ => self.is_permanent(),
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            user_id: -1,
            oidc_id: String::default(),
            username: String::default(),
            email: String::default(),
            admin_role: AdminRole::None,
            permission_by_forum_map: HashMap::new(),
            ban_status: BanStatus::None,
            ban_status_by_forum_map: HashMap::new(),
            timestamp: chrono::DateTime::default(),
            is_deleted: false,
        }
    }
}

impl User {
    fn check_forum_permissions(&self, forum_name: &str, req_permission_level: PermissionLevel) -> Result<(), AppError> {
        match self.permission_by_forum_map.get(forum_name).is_some_and(|permission_level| *permission_level >= req_permission_level) {
            true => Ok(()),
            false => Err(AppError::InsufficientPrivileges)
        }
    }
    
    pub fn check_admin_role(&self, req_admin_role: AdminRole) -> Result<(), AppError> {
        match self.admin_role >= req_admin_role {
            true => Ok(()),
            false => Err(AppError::InsufficientPrivileges),
        }
    }

    pub fn check_permissions(&self, forum_name: &str, req_permission_level: PermissionLevel) -> Result<(), AppError> {
        let has_admin_permission = self.admin_role.get_permission_level() >= req_permission_level;
        let has_forum_permission = self.check_forum_permissions(forum_name, req_permission_level).is_ok();
        match has_admin_permission || has_forum_permission {
            true => Ok(()),
            false => Err(AppError::InsufficientPrivileges)
        }
    }

    pub fn check_is_forum_leader(&self, forum_name: &str) -> Result<(), AppError> {
        self.check_forum_permissions(forum_name, PermissionLevel::Lead)
    }

    pub fn get_forum_permission_level(&self, forum_name: &str) -> PermissionLevel {
        max(self.admin_role.get_permission_level(), self.permission_by_forum_map.get(forum_name).cloned().unwrap_or(PermissionLevel::None))
    }
    
    pub fn check_can_publish(&self) -> Result<(), AppError> {
        match self.ban_status.is_active() {
            true => match self.ban_status {
                BanStatus::Until(timestamp) => Err(AppError::GlobalBanUntil(timestamp)),
                BanStatus::Permanent => Err(AppError::PermanentGlobalBan),
                BanStatus::None => Err(AppError::InternalServerError(String::from("User with BanStatus::None despite ban_status.is_active == true"))), // should never happen
            },
            false => Ok(())
        }
    }

    pub fn check_can_publish_on_forum(&self, forum_name: &str) -> Result<(), AppError> {
        self.check_can_publish()?;
        match self.ban_status_by_forum_map.get(forum_name) {
            Some(ban_status) if ban_status.is_active() => match ban_status {
                BanStatus::Until(timestamp) => Err(AppError::ForumBanUntil(*timestamp)),
                BanStatus::Permanent => Err(AppError::PermanentForumBan),
                BanStatus::None => Err(AppError::InternalServerError(String::from("User with forum BanStatus::None despite ban_status.is_active == true"))), // should never happen
            },
            _ => Ok(())
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use async_trait::async_trait;
    use axum_session_auth::Authentication;
    use sqlx::PgPool;

    use crate::errors::AppError;
    use crate::forum_management::UserBan;
    use crate::role::ssr::get_user_forum_role;
    use crate::role::UserForumRole;

    use super::*;

    #[derive(sqlx::FromRow, Clone, Debug, PartialEq)]
    pub struct SqlUser {
        pub user_id: i64,
        pub oidc_id: String,
        pub username: String,
        pub email: String,
        pub admin_role: AdminRole,
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub is_deleted: bool,
    }

    impl SqlUser {
        pub async fn get_by_username(
            username: &str,
            db_pool: &PgPool,
        ) -> Result<SqlUser, AppError> {
            let sql_user = sqlx::query_as!(
            SqlUser,
            "SELECT * FROM users WHERE username = $1",
            username,
        )
                .fetch_one(db_pool)
                .await?;

            Ok(sql_user)
        }
        pub async fn get_from_oidc_id(oidc_id: &String, db_pool: &PgPool) -> Result<SqlUser, AppError> {
            let sql_user = sqlx::query_as!(
                SqlUser,
                "SELECT * FROM users WHERE oidc_id = $1",
                oidc_id
            )
                .fetch_one(db_pool)
                .await?;

            Ok(sql_user)
        }
        pub fn into_user(
            self,
            user_role_vec: Vec<UserForumRole>,
            user_ban_vec: Vec<UserBan>,
        ) -> User {
            let mut permission_by_forum_map: HashMap<String, PermissionLevel> = HashMap::new();
            for user_forum_role in user_role_vec {
                permission_by_forum_map.insert(
                    user_forum_role.forum_name.clone(),
                    user_forum_role.permission_level,
                );
            }
            let mut global_ban_status = BanStatus::None;
            let mut ban_status_by_forum_map: HashMap<String, BanStatus> = HashMap::new();
            let current_timestamp = chrono::offset::Utc::now();
            for user_ban in user_ban_vec {
                let (ban_status, is_valid) = match user_ban.until_timestamp {
                    Some(until_timestamp) => (
                        BanStatus::Until(until_timestamp),
                        until_timestamp > current_timestamp,
                    ),
                    None => (BanStatus::Permanent, true),
                };
                if is_valid {
                    match user_ban.forum_name {
                        Some(forum_name) => {
                            match ban_status_by_forum_map.get_mut(&forum_name) {
                                Some(current_ban_status) => {
                                    if ban_status > *current_ban_status {
                                        *current_ban_status = ban_status;
                                    }
                                },
                                None => _ = ban_status_by_forum_map.insert(forum_name, ban_status),
                            };
                        },
                        None => {
                            if ban_status > global_ban_status {
                                global_ban_status = ban_status;
                            }
                        }
                    };
                }
            }

            User {
                user_id: self.user_id,
                oidc_id: self.oidc_id,
                username: self.username,
                email: self.email,
                admin_role: self.admin_role,
                permission_by_forum_map,
                ban_status: global_ban_status,
                ban_status_by_forum_map,
                timestamp: self.timestamp,
                is_deleted: self.is_deleted,
            }
        }
    }

    impl User {
        pub async fn get(user_id: i64, db_pool: &PgPool) -> Option<Self> {
            match sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE user_id = $1", user_id)
                .fetch_one(db_pool)
                .await
            {
                Ok(sql_user) => {
                    let user_forum_role_vec = load_user_forum_role_vec(sql_user.user_id, db_pool)
                        .await
                        .unwrap_or_default();
                    let user_ban_vec = load_user_ban_vec(sql_user.user_id, db_pool)
                        .await
                        .unwrap_or_default();
                    Some(sql_user.into_user(user_forum_role_vec, user_ban_vec))
                }
                Err(select_error) => {
                    log::debug!("User not found with error: {}", select_error);
                    None
                }
            }
        }

        pub async fn check_can_set_user_forum_role(
            &self,
            permission_level: PermissionLevel,
            user_id: i64,
            forum_name: &str,
            db_pool: &PgPool,
        ) -> Result<(), AppError> {
            match (self.admin_role, self.permission_by_forum_map.get(forum_name)) {
                (AdminRole::Admin, _) => Ok(()),
                (_, Some(own_level)) if *own_level >= PermissionLevel::Manage && *own_level > permission_level => {
                    match get_user_forum_role(user_id, forum_name, db_pool).await {
                        Err(AppError::NotFound) => Ok(()),
                        Ok(user_role) if *own_level > user_role.permission_level => Ok(()),
                        _ => Err(AppError::InsufficientPrivileges),
                    }
                },
                _ => Err(AppError::InsufficientPrivileges),
            }
        }
    }

    #[async_trait]
    impl Authentication<User, i64, PgPool> for User {
        async fn load_user(user_id: i64, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
            let pool = pool.ok_or(anyhow::anyhow!("Cannot get DB pool"))?;

            User::get(user_id, pool)
                .await
                .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
        }

        fn is_authenticated(&self) -> bool {
            true
        }

        fn is_active(&self) -> bool {
            true
        }

        fn is_anonymous(&self) -> bool {
            false
        }
    }

    pub async fn get_matching_username_set(
        username_prefix: &str,
        limit: i64,
        db_pool: &PgPool,
    ) -> Result<BTreeSet<String>, AppError> {
        let username_vec = sqlx::query!(
            "SELECT username FROM users WHERE username LIKE $1 ORDER BY username LIMIT $2",
            format!("{username_prefix}%"),
            limit,
        )
            .fetch_all(db_pool)
            .await?;

        let mut username_set = BTreeSet::<String>::new();

        for row in username_vec {
            username_set.insert(row.username);
        }

        Ok(username_set)
    }

    pub async fn create_user(
        oidc_id: &str,
        username: &str,
        email: &str,
        db_pool: &PgPool,
    ) -> Result<SqlUser, AppError> {
        log::debug!("Create new user {username}");
        let sql_user = sqlx::query_as!(
            SqlUser,
            "INSERT INTO users (oidc_id, username, email) VALUES ($1, $2, $3) RETURNING *",
            oidc_id,
            username,
            email,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(sql_user)
    }

    async fn load_user_forum_role_vec(
        user_id: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<UserForumRole>, AppError> {
        let user_forum_role_vec = sqlx::query_as!(
            UserForumRole,
            "SELECT *
            FROM user_forum_roles
            WHERE user_id = $1",
            user_id
        )
            .fetch_all(db_pool)
            .await?;
        log::trace!("User roles: {:?}", user_forum_role_vec);
        Ok(user_forum_role_vec)
    }

    async fn load_user_ban_vec(user_id: i64, db_pool: &PgPool) -> Result<Vec<UserBan>, AppError> {
        let user_ban_vec = sqlx::query_as!(
            UserBan,
            "SELECT * FROM user_bans WHERE user_id = $1 AND (until_timestamp > CURRENT_TIMESTAMP or until_timestamp IS NULL)",
            user_id
        )
            .fetch_all(db_pool)
            .await?;
        log::trace!("User bans: {:?}", user_ban_vec);
        Ok(user_ban_vec)
    }

    #[cfg(test)]
    mod tests {
        use std::ops::Add;

        use chrono::Days;

        use super::*;

        #[test]
        fn test_sql_user_into_user() {
            let past_timestamp = chrono::DateTime::from_timestamp_nanos(0);
            let future_timestamp = chrono::offset::Utc::now().add(Days::new(1));
            let sql_user = SqlUser {
                user_id: 0,
                oidc_id: String::from("a"),
                username: String::from("b"),
                email: String::from("c"),
                admin_role: AdminRole::None,
                timestamp: chrono::DateTime::from_timestamp_nanos(0),
                is_deleted: false,
            };
            let user_forum_role_vec = vec![
                UserForumRole {
                    role_id: 0,
                    user_id: 0,
                    username: String::from("b"),
                    forum_id: 0,
                    forum_name: String::from("0"),
                    permission_level: PermissionLevel::Moderate,
                    grantor_id: 0,
                    timestamp: past_timestamp,
                },
                UserForumRole {
                    role_id: 0,
                    user_id: 0,
                    username: String::from("b"),
                    forum_id: 1,
                    forum_name: String::from("1"),
                    permission_level: PermissionLevel::Lead,
                    grantor_id: 0,
                    timestamp: past_timestamp,
                },
            ];
            let user_ban_vec = vec![
                UserBan {
                    ban_id: 0,
                    user_id: 0,
                    username: String::from("b"),
                    forum_id: None,
                    forum_name: None,
                    moderator_id: 0,
                    until_timestamp: Some(past_timestamp),
                    create_timestamp: Default::default(),
                },
                UserBan {
                    ban_id: 1,
                    user_id: 0,
                    username: String::from("b"),
                    forum_id: Some(0),
                    forum_name: Some(String::from("a")),
                    moderator_id: 0,
                    until_timestamp: Some(past_timestamp),
                    create_timestamp: Default::default(),
                },
                UserBan {
                    ban_id: 2,
                    user_id: 0,
                    username: String::from("b"),
                    forum_id: Some(1),
                    forum_name: Some(String::from("b")),
                    moderator_id: 0,
                    until_timestamp: Some(future_timestamp),
                    create_timestamp: Default::default(),
                },
                UserBan {
                    ban_id: 3,
                    user_id: 0,
                    username: String::from("b"),
                    forum_id: Some(2),
                    forum_name: Some(String::from("c")),
                    moderator_id: 0,
                    until_timestamp: None,
                    create_timestamp: Default::default(),
                },
            ];
            let user_1 = sql_user.clone().into_user(user_forum_role_vec.clone(), user_ban_vec);
            assert_eq!(user_1.user_id, 0);
            assert_eq!(user_1.oidc_id, "a");
            assert_eq!(user_1.username, "b");
            assert_eq!(user_1.email, "c");
            assert_eq!(user_1.admin_role, AdminRole::None);
            assert_eq!(user_1.timestamp, chrono::DateTime::from_timestamp_nanos(0));
            assert_eq!(user_1.is_deleted, false);
            assert_eq!(user_1.permission_by_forum_map[&String::from("0")], PermissionLevel::Moderate);
            assert_eq!(user_1.permission_by_forum_map[&String::from("1")], PermissionLevel::Lead);
            assert_eq!(user_1.ban_status, BanStatus::None);
            assert_eq!(user_1.ban_status_by_forum_map.get(&String::from("a")), None);
            assert_eq!(
                *user_1.ban_status_by_forum_map.get(&String::from("b")).expect("User should have ban for forum 'b'."),
                BanStatus::Until(future_timestamp)
            );
            assert_eq!(
                *user_1.ban_status_by_forum_map.get(&String::from("c")).expect("User should have ban for forum 'c'."),
                BanStatus::Permanent
            );

            let user_2_ban_vec = vec![UserBan {
                ban_id: 3,
                user_id: 0,
                username: String::from("b"),
                forum_id: None,
                forum_name: None,
                moderator_id: 0,
                until_timestamp: Some(future_timestamp),
                create_timestamp: Default::default(),
            }];
            let user_2 = sql_user.into_user(user_forum_role_vec, user_2_ban_vec);
            assert_eq!(user_2.ban_status, BanStatus::Until(future_timestamp));
        }
    }
}

#[server]
pub async fn get_matching_username_set(
    username_prefix: String,
) -> Result<BTreeSet<String>, ServerFnError> {
    let db_pool = get_db_pool()?;
    let username_set = ssr::get_matching_username_set(&username_prefix, USER_FETCH_LIMIT, &db_pool).await?;
    Ok(username_set)
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::Days;

    use super::*;

    fn get_user_permission_map() -> HashMap<String, PermissionLevel> {
        HashMap::from([
            (String::from("a"), PermissionLevel::None),
            (String::from("b"), PermissionLevel::Moderate),
            (String::from("c"), PermissionLevel::Ban),
            (String::from("d"), PermissionLevel::Manage),
            (String::from("e"), PermissionLevel::Lead),
        ])
    }

    #[test]
    fn test_ban_status_is_permanent() {
        let ban_status_none = BanStatus::None;
        let ban_status_until = BanStatus::Until(chrono::DateTime::from_timestamp_nanos(0));
        let ban_status_permanent = BanStatus::Permanent;
        assert_eq!(ban_status_none.is_permanent(), false);
        assert_eq!(ban_status_until.is_permanent(), false);
        assert_eq!(ban_status_permanent.is_permanent(), true);
    }

    #[test]
    fn test_ban_status_is_active() {
        let ban_status_none = BanStatus::None;
        let ban_status_until_past = BanStatus::Until(chrono::DateTime::from_timestamp_nanos(0));
        let ban_status_until_future = BanStatus::Until(chrono::offset::Utc::now().add(Days::new(1)));
        let ban_status_permanent = BanStatus::Permanent;
        assert_eq!(ban_status_none.is_active(), false);
        assert_eq!(ban_status_until_past.is_active(), false);
        assert_eq!(ban_status_until_future.is_active(), true);
        assert_eq!(ban_status_permanent.is_active(), true);
    }

    #[test]
    fn test_user_check_admin_role() {
        let mut user = User::default();
        assert_eq!(user.check_admin_role(AdminRole::None), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Moderator), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_admin_role(AdminRole::Admin), Err(AppError::InsufficientPrivileges));
        user.admin_role = AdminRole::Moderator;
        assert_eq!(user.check_admin_role(AdminRole::None), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Moderator), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Admin), Err(AppError::InsufficientPrivileges));
        user.admin_role = AdminRole::Admin;
        assert_eq!(user.check_admin_role(AdminRole::None), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Moderator), Ok(()));
        assert_eq!(user.check_admin_role(AdminRole::Admin), Ok(()));
    }

    #[test]
    fn test_user_check_permissions() {
        let mut user = User::default();
        user.permission_by_forum_map = get_user_permission_map();
        assert_eq!(user.check_permissions("a", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::None), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Moderate), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("b", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Moderate), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Ban), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("b", PermissionLevel::Ban), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("c", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Ban), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Manage), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("b", PermissionLevel::Manage), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("c", PermissionLevel::Manage), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("d", PermissionLevel::Manage), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Manage), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("b", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("c", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("d", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("e", PermissionLevel::Lead), Ok(()));
        
        user.admin_role = AdminRole::Moderator;

        assert_eq!(user.check_permissions("a", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::None), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Moderate), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Ban), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Manage), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("b", PermissionLevel::Manage), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("c", PermissionLevel::Manage), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("d", PermissionLevel::Manage), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Manage), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("b", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("c", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("d", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_permissions("e", PermissionLevel::Lead), Ok(()));

        user.admin_role = AdminRole::Admin;

        assert_eq!(user.check_permissions("a", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::None), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::None), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Moderate), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Moderate), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Ban), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Ban), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Manage), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::Manage), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::Manage), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Manage), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Manage), Ok(()));

        assert_eq!(user.check_permissions("a", PermissionLevel::Lead), Ok(()));
        assert_eq!(user.check_permissions("b", PermissionLevel::Lead), Ok(()));
        assert_eq!(user.check_permissions("c", PermissionLevel::Lead), Ok(()));
        assert_eq!(user.check_permissions("d", PermissionLevel::Lead), Ok(()));
        assert_eq!(user.check_permissions("e", PermissionLevel::Lead), Ok(()));
        
        let mut admin = User::default();
        admin.admin_role = AdminRole::Moderator;
        assert_eq!(admin.check_permissions("a", PermissionLevel::None), Ok(()));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Moderate), Ok(()));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Ban), Ok(()));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Manage), Err(AppError::InsufficientPrivileges));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Lead), Err(AppError::InsufficientPrivileges));
        admin.admin_role = AdminRole::Admin;
        assert_eq!(admin.check_permissions("a", PermissionLevel::None), Ok(()));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Moderate), Ok(()));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Ban), Ok(()));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Manage), Ok(()));
        assert_eq!(admin.check_permissions("a", PermissionLevel::Lead), Ok(()));
    }

    #[test]
    fn test_user_check_is_forum_leader() {
        let mut user = User::default();
        user.permission_by_forum_map = get_user_permission_map();
        assert_eq!(user.check_is_forum_leader("a"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("b"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("c"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("d"), Err(AppError::InsufficientPrivileges));
        assert_eq!(user.check_is_forum_leader("e"), Ok(()));
        let mut admin = User::default();
        admin.admin_role = AdminRole::Moderator;
        assert_eq!(admin.check_is_forum_leader("a"), Err(AppError::InsufficientPrivileges));
        admin.admin_role = AdminRole::Admin;
        assert_eq!(admin.check_is_forum_leader("a"), Err(AppError::InsufficientPrivileges));
    }

    #[test]
    fn test_user_get_forum_permission_level() {
        let mut user = User::default();
        user.permission_by_forum_map = get_user_permission_map();
        assert_eq!(user.get_forum_permission_level("missing"), PermissionLevel::None);
        assert_eq!(user.get_forum_permission_level("a"), PermissionLevel::None);
        assert_eq!(user.get_forum_permission_level("b"), PermissionLevel::Moderate);
        assert_eq!(user.get_forum_permission_level("c"), PermissionLevel::Ban);
        assert_eq!(user.get_forum_permission_level("d"), PermissionLevel::Manage);
        assert_eq!(user.get_forum_permission_level("e"), PermissionLevel::Lead);
    }

    #[test]
    fn test_user_check_can_publish() {
        let past_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let future_timestamp = chrono::offset::Utc::now().add(Days::new(1));
        let mut user = User::default();
        assert_eq!(user.check_can_publish(), Ok(()));
        user.ban_status = BanStatus::Until(past_timestamp);
        assert_eq!(user.check_can_publish(), Ok(()));
        user.ban_status = BanStatus::Until(future_timestamp);
        assert_eq!(user.check_can_publish(), Err(AppError::GlobalBanUntil(future_timestamp)));
        user.ban_status = BanStatus::Permanent;
        assert_eq!(user.check_can_publish(), Err(AppError::PermanentGlobalBan));
    }

    #[test]
    fn test_user_check_can_publish_on_forum() {
        let past_timestamp = chrono::DateTime::from_timestamp_nanos(0);
        let future_timestamp = chrono::offset::Utc::now().add(Days::new(1));
        let mut user = User::default();
        user.ban_status_by_forum_map = HashMap::from([
            (String::from("a"), BanStatus::None),
            (String::from("b"), BanStatus::Until(past_timestamp)),
            (String::from("c"), BanStatus::Until(future_timestamp)),
            (String::from("d"), BanStatus::Permanent),
        ]);
        assert_eq!(user.check_can_publish_on_forum("a"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("b"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::ForumBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::PermanentForumBan));
        user.ban_status = BanStatus::Until(past_timestamp);
        assert_eq!(user.check_can_publish_on_forum("a"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("b"), Ok(()));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::ForumBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::PermanentForumBan));
        user.ban_status = BanStatus::Until(future_timestamp);
        assert_eq!(user.check_can_publish_on_forum("a"), Err(AppError::GlobalBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("b"), Err(AppError::GlobalBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::GlobalBanUntil(future_timestamp)));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::GlobalBanUntil(future_timestamp)));
        user.ban_status = BanStatus::Permanent;
        assert_eq!(user.check_can_publish_on_forum("a"), Err(AppError::PermanentGlobalBan));
        assert_eq!(user.check_can_publish_on_forum("b"), Err(AppError::PermanentGlobalBan));
        assert_eq!(user.check_can_publish_on_forum("c"), Err(AppError::PermanentGlobalBan));
        assert_eq!(user.check_can_publish_on_forum("d"), Err(AppError::PermanentGlobalBan));
    }
}