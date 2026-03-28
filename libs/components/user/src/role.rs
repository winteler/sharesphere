use std::str::FromStr;

use leptos::prelude::*;
use leptos_fluent::{move_tr};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use strum_macros::{Display, EnumString, IntoStaticStr};

use sharesphere_core_common::errors::AppError;
use sharesphere_core_common::form::LabeledFormCheckbox;
use sharesphere_core_common::unpack::SuspenseUnpack;

use crate::user::UserState;

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
                Some(user) if user.check_sphere_permissions_by_name(&sphere_name.read(), permission_level).is_ok() => {
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
    #[prop(default = "is_pinned")]
    name: &'static str,
    #[prop(default = false)]
    value: bool,
) -> impl IntoView {
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Moderate>
            <LabeledFormCheckbox name label=move_tr!("pinned") value/>
        </AuthorizedShow>
    }
}