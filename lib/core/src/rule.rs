use std::str::FromStr;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_fluent::{move_tr};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};

use sharesphere_utils::errors::{AppError, ErrorDisplay};

#[cfg(feature = "ssr")]
use {
    sharesphere_auth::{
        auth::ssr::check_user,
        session::ssr::get_db_pool,
    },
    sharesphere_utils::checks::{check_sphere_name, check_string_length},
    sharesphere_utils::constants::{MAX_MOD_MESSAGE_LENGTH, MAX_TITLE_LENGTH},
    sharesphere_utils::editor::ssr::get_html_and_markdown_strings,
};
use sharesphere_utils::icons::LoadingIcon;
use sharesphere_utils::widget::{Collapse, ContentBody, TitleCollapse};
use crate::state::GlobalState;

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
pub enum BaseRule {
    BeRespectful,
    RespectRules,
    NoIllegalContent,
    PlatformIntegrity,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id: i64,
    pub rule_key: i64, // business id to track rule across updates
    pub sphere_id: Option<i64>,
    pub priority: i16,
    pub title: String,
    pub description: String,
    pub markdown_description: Option<String>,
    pub user_id: i64,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub delete_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl BaseRule {
    pub fn get_localized_title(self) -> Signal<String> {
        match self {
            BaseRule::BeRespectful => move_tr!("rule-respectful-title"),
            BaseRule::RespectRules => move_tr!("rule-respect-rules-title"),
            BaseRule::NoIllegalContent => move_tr!("rule-no-illegal-content-title"),
            BaseRule::PlatformIntegrity => move_tr!("rule-platform-integrity-title"),
        }
    }

    pub fn get_localized_description(self) -> Signal<String> {
        match self {
            BaseRule::BeRespectful => move_tr!("rule-respectful-description"),
            BaseRule::RespectRules => move_tr!("rule-respect-rules-description"),
            BaseRule::NoIllegalContent => move_tr!("rule-no-illegal-content-description"),
            BaseRule::PlatformIntegrity => move_tr!("rule-platform-integrity-description"),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;
    use sharesphere_auth::role::{PermissionLevel};
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

    pub async fn get_rule_vec(
        sphere_name: Option<&str>,
        db_pool: &PgPool,
    ) -> Result<Vec<Rule>, AppError> {
        let sphere_rule_vec = sqlx::query_as!(
            Rule,
            "SELECT r.* FROM rules r
            LEFT JOIN spheres s ON s.sphere_id = r.sphere_id
            WHERE
                COALESCE(s.sphere_name, $1) IS NOT DISTINCT FROM $1 AND
                r.delete_timestamp IS NULL
            ORDER BY s.sphere_name NULLS FIRST, r.priority, r.create_timestamp",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(sphere_rule_vec)
    }

    pub async fn add_rule(
        sphere_name: &str,
        priority: i16,
        title: &str,
        description: &str,
        markdown_description: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        user.check_sphere_permissions_by_name(sphere_name, PermissionLevel::Manage)?;

        sqlx::query!(
            "UPDATE rules
             SET priority = priority + 1
             WHERE sphere_id = (
                    SELECT sphere_id FROM spheres WHERE sphere_name = $1
                 ) AND priority >= $2 AND delete_timestamp IS NULL",
            sphere_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        let rule = sqlx::query_as!(
            Rule,
            "INSERT INTO rules
            (sphere_id, priority, title, description, markdown_description, user_id)
            VALUES (
                (SELECT sphere_id FROM spheres WHERE sphere_name = $1),
                $2, $3, $4, $5, $6
            ) RETURNING *",
            sphere_name,
            priority,
            title,
            description,
            markdown_description,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(rule)
    }

    pub async fn update_rule(
        sphere_name: &str,
        current_priority: i16,
        priority: i16,
        title: &str,
        description: &str,
        markdown_description: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Rule, AppError> {
        user.check_sphere_permissions_by_name(sphere_name, PermissionLevel::Manage)?;

        let current_rule = sqlx::query_as!(
            Rule,
            "UPDATE rules
             SET delete_timestamp = NOW()
             WHERE sphere_id = (
                    SELECT sphere_id FROM spheres where sphere_name = $1
                ) AND priority = $2 AND delete_timestamp IS NULL
             RETURNING *",
            sphere_name,
            current_priority,
        ).fetch_one(db_pool).await?;

        if priority > current_priority {
            sqlx::query!(
                "UPDATE rules
                SET priority = priority - 1
                WHERE sphere_id = (
                        SELECT sphere_id FROM spheres WHERE sphere_name = $1
                    ) AND priority BETWEEN $2 AND $3 AND delete_timestamp IS NULL",
                sphere_name,
                current_priority,
                priority,
            ).execute(db_pool).await?;
        } else if priority < current_priority {
            sqlx::query!(
                "UPDATE rules
                SET priority = priority + 1
                WHERE sphere_id = (
                        SELECT sphere_id FROM spheres WHERE sphere_name = $1
                    ) AND priority BETWEEN $3 AND $2 AND delete_timestamp IS NULL",
                sphere_name,
                current_priority,
                priority,
            ).execute(db_pool).await?;
        }

        let new_rule = sqlx::query_as!(
            Rule,
            "INSERT INTO rules
            (rule_key, sphere_id, priority, title, description, markdown_description, user_id)
            VALUES (
                $1,
                (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                $3, $4, $5, $6, $7
            ) RETURNING *",
            current_rule.rule_key,
            sphere_name,
            priority,
            title,
            description,
            markdown_description,
            user.user_id,
        ).fetch_one(db_pool).await?;

        Ok(new_rule)
    }

    pub async fn remove_rule(
        sphere_name: &str,
        priority: i16,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<(), AppError> {
        user.check_sphere_permissions_by_name(sphere_name, PermissionLevel::Manage)?;

        sqlx::query!(
            "UPDATE rules
             SET delete_timestamp = NOW()
             WHERE sphere_id = (
                    SELECT sphere_id FROM spheres WHERE sphere_name = $1
                ) AND priority = $2 AND delete_timestamp IS NULL",
            sphere_name,
            priority,
        )
            .execute(db_pool)
            .await?;

        sqlx::query!(
            "UPDATE rules
             SET priority = priority - 1
             WHERE sphere_id = (
                    SELECT sphere_id FROM spheres WHERE sphere_name = $1
                ) AND priority > $2 AND delete_timestamp IS NULL",
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
pub async fn get_rule_vec(
    sphere_name: Option<String>
) -> Result<Vec<Rule>, AppError> {
    if let Some(sphere_name) = &sphere_name {
        check_sphere_name(&sphere_name)?;
    }
    let db_pool = get_db_pool()?;
    let rule_vec = ssr::get_rule_vec(sphere_name.as_deref(), &db_pool).await?;
    Ok(rule_vec)
}

#[server]
pub async fn add_rule(
    sphere_name: String,
    priority: i16,
    title: String,
    description: String,
    is_markdown: bool,
) -> Result<Rule, AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&title, "Title", MAX_TITLE_LENGTH as usize, false)?;
    check_string_length(&description, "Description", MAX_MOD_MESSAGE_LENGTH, true)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let (description, markdown_description) = get_html_and_markdown_strings(description, is_markdown).await?;

    let rule = ssr::add_rule(
        sphere_name.as_ref(),
        priority,
        &title,
        &description,
        markdown_description.as_deref(),
        &user,
        &db_pool
    ).await?;

    Ok(rule)
}

#[server]
pub async fn update_rule(
    sphere_name: String,
    current_priority: i16,
    priority: i16,
    title: String,
    description: String,
    is_markdown: bool,
) -> Result<Rule, AppError> {
    check_sphere_name(&sphere_name)?;
    check_string_length(&title, "Title", MAX_TITLE_LENGTH as usize, false)?;
    check_string_length(&description, "Description", MAX_MOD_MESSAGE_LENGTH, true)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let (description, markdown_description) = get_html_and_markdown_strings(description, is_markdown).await?;

    let rule = ssr::update_rule(
        sphere_name.as_ref(),
        current_priority,
        priority,
        &title,
        &description,
        markdown_description.as_deref(),
        &user,
        &db_pool
    ).await?;

    Ok(rule)
}

#[server]
pub async fn remove_rule(
    sphere_name: String,
    priority: i16,
) -> Result<(), AppError> {
    check_sphere_name(&sphere_name)?;
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    ssr::remove_rule(sphere_name.as_ref(), priority, &user, &db_pool).await?;
    Ok(())
}

/// List of collapsable rules
#[component]
pub fn BaseRuleList() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <TitleCollapse title=move_tr!("rules")>
            <Suspense fallback=move || view! { <LoadingIcon/> }.into_any()>
            {
                move || Suspend::new(async move {
                    match &state.base_rules.await {
                        Ok(rule_vec) => Either::Left(view!{
                            <RuleList rule_vec=rule_vec.clone()/>
                        }),
                        Err(e) => Either::Right(view! { <ErrorDisplay error=e.clone()/> } ),
                    }
                })
            }
            </Suspense>
        </TitleCollapse>
    }
}

/// List of collapsable rules
#[component]
pub fn RuleList(
    rule_vec: Vec<Rule>,
) -> impl IntoView {
    let rule_elems = rule_vec.into_iter().enumerate().map(|(index, rule)| {
        let is_markdown = rule.markdown_description.is_some();
        let is_base_rule = rule.sphere_id.is_none();

        let title = match is_base_rule {
            true => BaseRule::from_str(&rule.title).unwrap().get_localized_title(),
            false => {
                let title = rule.title.clone();
                Signal::derive(move || title.clone())
            },
        };
        let description = match is_base_rule {
            true => BaseRule::from_str(&rule.title).unwrap().get_localized_description(),
            false => Signal::derive(move || rule.description.clone()),
        };
        let title_view = move || view! {
            <div class="flex gap-2">
                <div class="text-semibold">{format!("{}.", index+1)}</div>
                <div class="text-left text-semibold">{title}</div>
            </div>
        };
        view! {
            <Collapse
                title_view
                is_open=false
            >
                <div class="pl-1 pb-3">
                    <ContentBody body=description is_markdown/>
                </div>
            </Collapse>
        }
    }).collect_view();

    view! {
        <div class="flex flex-col pl-1 pt-1 gap-1">
        {rule_elems}
        </div>
    }
}