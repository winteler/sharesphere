use std::env;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_query_map};
use leptos_router::params::Params;
use web_sys::MouseEvent;

use sharesphere_utils::errors::AppError;
use sharesphere_utils::icons::LoadingIcon;
use sharesphere_utils::unpack::SuspenseUnpack;

use crate::user::{User, UserState};


/// Guard for a component requiring a login. If the user is logged in, the children of this component will be rendered
/// Otherwise, it will be replaced by a form/button with the same appearance redirecting to a login screen.
#[component]
pub fn LoginGuardButton<
    F: Fn(&User) -> IV + Clone + Send + Sync + 'static,
    IV: IntoView + 'static,
>(
    #[prop(default = "")]
    login_button_class: &'static str,
    #[prop(into)]
    login_button_content: ViewFn,
    #[prop(into, default=use_location().pathname.into())]
    redirect_path: Signal<String>,
    #[prop(default = "loading-icon-size")]
    loading_icon_class: &'static str,
    children: F,
) -> impl IntoView {
    let user_state = expect_context::<UserState>();
    let children = StoredValue::new(children);
    let login_button_content = StoredValue::new(login_button_content);

    view! {
        <Transition fallback=move || view! { <LoadingIcon class=loading_icon_class/> }>
        {
            move || Suspend::new(async move {
                match &user_state.user.await {
                    Ok(Some(user)) => children.with_value(|children| children(user)).into_any(),
                    _ => {
                        let login_button_view = login_button_content.with_value(|content| content.run());
                        view! { <LoginButton class=login_button_class redirect_path>{login_button_view}</LoginButton> }.into_any()
                    },
                }
            })
        }
        </Transition>
    }.into_any()
}

/// Login guarded button component. If the user is logged in, a button with the given class and action will be rendered.
/// Otherwise, the button will redirect the user to a login screen.
#[component]
pub fn LoginGuardedButton<A, IV>(
    #[prop(into)]
    button_class: Signal<&'static str>,
    button_action: A,
    children: TypedChildrenFn<IV>,
    #[prop(default = "loading-icon-size")]
    loading_icon_class: &'static str,
) -> impl IntoView
where
    A: Fn(MouseEvent) -> () + Clone + Send + Sync + 'static,
    IV: IntoView + 'static
{
    let user_state = expect_context::<UserState>();
    let children = StoredValue::new(children.into_inner());
    let button_action = StoredValue::new(button_action);
    view! {
        <Transition fallback=move || view! { <LoadingIcon class=loading_icon_class/> }>
        {
            move || Suspend::new(async move {
                let children_view = children.with_value(|children| children());
                match &user_state.user.await {
                    Ok(Some(_)) => view! {
                        <button
                            class=button_class
                            aria-haspopup="dialog"
                            on:click=button_action.get_value()
                        >
                            {children_view}
                        </button>
                    }.into_any(),
                    _ => view! { <LoginButton class=button_class redirect_path=use_location().pathname>{children_view}</LoginButton> }.into_any(),
                }
            })
        }
        </Transition>
    }
}

#[component]
fn LoginButton(
    #[prop(into)]
    class: Signal<&'static str>,
    #[prop(into)]
    redirect_path: Signal<String>,
    children: Children,
) -> impl IntoView {
    let user_state = expect_context::<UserState>();

    view! {
        <ActionForm action=user_state.login_action attr:class="flex items-center">
            <input type="text" name="redirect_url" class="hidden" value=redirect_path/>
            <button type="submit" class=class>
                {children()}
            </button>
        </ActionForm>
    }.into_any()
}

/// Auth callback component
#[component]
pub fn AuthCallback() -> impl IntoView {
    let query = use_query_map();
    let code = move || query.read_untracked().get("code").unwrap_or_default().to_string();
    let auth_resource = Resource::new_blocking(
        || (),
        move |_| {
            log::trace!("Authenticate user.");
            authenticate_user(code())
        }
    );

    view! {
        <SuspenseUnpack
            resource=auth_resource
            let:_auth_result
        >
            {
                log::debug!("Authenticated successfully");
            }
        </SuspenseUnpack>
    }.into_any()
}
