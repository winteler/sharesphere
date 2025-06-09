use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use sharesphere_utils::errors::AppError;

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::check_user,
        session::ssr::get_db_pool,
    },
};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id: i64,
    pub rule_key: i64, // business id to track rule across updates
    pub sphere_id: Option<i64>,
    pub sphere_name: Option<String>,
    pub priority: i16,
    pub title: String,
    pub description: String,
    pub markdown_description: Option<String>,
    pub user_id: i64,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::{AdminRole, PermissionLevel};
    use sharesphere_auth::user::User;
    use sharesphere_utils::errors::AppError;
    use crate::rule::Rule;

    pub async fn load_rule_by_id(
        rule_id: i64,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        let rule = sqlx::query_as!(
            Rule,
            "SELECT * FROM rules
            WHERE rule_id = $1",
            rule_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(rule)
    }

    pub async fn get_sphere_rule_vec(
        sphere_name: &str,
        db_pool: &PgPool,
    ) -> Result<Vec<Rule>, AppError> {
        let sphere_rule_vec = sqlx::query_as!(
            Rule,
            "SELECT * FROM rules
            WHERE COALESCE(sphere_name, $1) = $1 AND delete_timestamp IS NULL
            ORDER BY sphere_name NULLS FIRST, priority, create_timestamp",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_rule_vec)
    }

    pub async fn add_rule(
        sphere_name: Option<&str>,
        priority: i16,
        title: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        match sphere_name {
            Some(sphere_name) => user.check_permissions(sphere_name, PermissionLevel::Manage)?,
            None => user.check_admin_role(AdminRole::Admin)?,
        };

        sqlx::query!(
            "UPDATE rules
             SET priority = priority + 1
             WHERE sphere_name IS NOT DISTINCT FROM $1 AND priority >= $2 AND delete_timestamp IS NULL",
            sphere_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        let rule = sqlx::query_as!(
            Rule,
            "INSERT INTO rules
            (sphere_id, sphere_name, priority, title, description, user_id)
            VALUES (
                (SELECT sphere_id FROM spheres WHERE sphere_name = $1),
                $1, $2, $3, $4, $5
            ) RETURNING *",
            sphere_name,
            priority,
            title,
            description,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(rule)
    }

    pub async fn update_rule(
        sphere_name: Option<&str>,
        current_priority: i16,
        priority: i16,
        title: &str,
        description: &str,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        match sphere_name {
            Some(sphere_name) => user.check_permissions(sphere_name, PermissionLevel::Manage)?,
            None => user.check_admin_role(AdminRole::Admin)?,
        };

        let current_rule = sqlx::query_as!(
            Rule,
            "UPDATE rules
             SET delete_timestamp = CURRENT_TIMESTAMP
             WHERE sphere_name IS NOT DISTINCT FROM $1 AND priority = $2 AND delete_timestamp IS NULL
             RETURNING *",
            sphere_name,
            current_priority,
        )
            .fetch_one(db_pool)
            .await?;

        if priority > current_priority {
            sqlx::query!(
                "UPDATE rules
                SET priority = priority - 1
                WHERE sphere_name IS NOT DISTINCT FROM $1 AND priority BETWEEN $2 AND $3 AND delete_timestamp IS NULL",
                sphere_name,
                current_priority,
                priority,
            )
                .execute(db_pool)
                .await?;
        } else if priority < current_priority {
            sqlx::query!(
                "UPDATE rules
                SET priority = priority + 1
                WHERE sphere_name IS NOT DISTINCT FROM $1 AND priority BETWEEN $3 AND $2 AND delete_timestamp IS NULL",
                sphere_name,
                current_priority,
                priority,
            )
                .execute(db_pool)
                .await?;
        }

        let new_rule = sqlx::query_as!(
            Rule,
            "INSERT INTO rules
            (rule_key, sphere_id, sphere_name, priority, title, description, user_id)
            VALUES (
                $1,
                (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                $2, $3, $4, $5, $6
            ) RETURNING *",
            current_rule.rule_key,
            sphere_name,
            priority,
            title,
            description,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(new_rule)
    }

    pub async fn remove_rule(
        sphere_name: Option<&str>,
        priority: i16,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        match sphere_name {
            Some(sphere_name) => user.check_permissions(sphere_name, PermissionLevel::Manage)?,
            None => user.check_admin_role(AdminRole::Admin)?,
        };

        sqlx::query!(
            "UPDATE rules
             SET delete_timestamp = CURRENT_TIMESTAMP
             WHERE sphere_name IS NOT DISTINCT FROM $1 AND priority = $2 AND delete_timestamp IS NULL",
            sphere_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        sqlx::query!(
            "UPDATE rules
             SET priority = priority - 1
             WHERE sphere_name IS NOT DISTINCT FROM $1 AND priority > $2 AND delete_timestamp IS NULL",
            sphere_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        Ok(())
    }
}

#[server]
pub async fn get_rule_by_id(
    rule_id: i64
) -> Result<Rule, AppError> {
    let db_pool = get_db_pool()?;
    let rule = ssr::load_rule_by_id(rule_id, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn get_sphere_rule_vec(
    sphere_name: String
) -> Result<Vec<Rule>, AppError> {
    let db_pool = get_db_pool()?;
    let rule_vec = ssr::get_sphere_rule_vec(&sphere_name, &db_pool).await?;
    Ok(rule_vec)
}

#[server]
pub async fn add_rule(
    sphere_name: Option<String>,
    priority: i16,
    title: String,
    description: String,
) -> Result<Rule, AppError> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let rule = ssr::add_rule(sphere_name.as_ref().map(String::as_str), priority, &title, &description, &user, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn update_rule(
    sphere_name: Option<String>,
    current_priority: i16,
    priority: i16,
    title: String,
    description: String,
) -> Result<Rule, AppError> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let rule = ssr::update_rule(sphere_name.as_ref().map(String::as_str), current_priority, priority, &title, &description, &user, &db_pool).await?;
    Ok(rule)
}

#[server]
pub async fn remove_rule(
    sphere_name: Option<String>,
    priority: i16,
) -> Result<(), AppError> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::remove_rule(sphere_name.as_deref(), priority, &user, &db_pool).await?;
    Ok(())
}