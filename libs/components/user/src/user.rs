use std::cmp::max;
use std::collections::{HashMap};
use std::default::Default;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::{NsfwIcon, UserIcon};

use crate::auth::Login;
use crate::role::{AdminRole, PermissionLevel};


/// Component to display a user header
#[component]
pub fn UserHeaderWidget(
    user_header: UserHeader,
) -> impl IntoView {
    view! {
        <div class="flex gap-1.5 items-center text-sm">
            <UserIcon/>
            {user_header.username}
            {
                match user_header.is_nsfw {
                    true => Some(view! { <NsfwIcon/> }),
                    false => None,
                }
            }
        </div>
    }.into_any()
}