use std::str::FromStr;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_fluent::{move_tr};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, IntoStaticStr};

use sharesphere_utils::errors::{AppError, ErrorDisplay};
use sharesphere_utils::icons::LoadingIcon;
use sharesphere_utils::widget::{Collapse, ContentBody, TitleCollapse};
use crate::state::GlobalState;


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
        let is_sphere_rule = rule.sphere_id.is_some();

        let title = get_rule_title(&rule.title, is_sphere_rule);
        let description = get_rule_description(&rule.title, &rule.description, is_sphere_rule);
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