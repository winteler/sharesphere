use leptos::prelude::*;
use leptos::server_fn::codec::PostUrl;
use leptos::server_fn::Http;
use leptos_fluent::move_tr;
use leptos_router::hooks::{use_location, use_navigate};
use leptos_router::NavigateOptions;
use serde::de::DeserializeOwned;
use server_fn::client::Client;
use server_fn::request::ClientReq;
use server_fn::ServerFn;
use web_sys::FormData;

use sharesphere_utils::errors::AppError;
use sharesphere_utils::form::LabeledSignalCheckbox;
use sharesphere_utils::icons::{AuthErrorIcon, AuthorIcon, DeleteIcon, LoadingIcon, ModeratorAuthorIcon, SelfAuthorIcon};
use sharesphere_utils::routes::{get_profile_path};
use sharesphere_utils::unpack::ActionError;
use sharesphere_utils::widget::{ModalDialog, ModalFormButtons};
use crate::auth::LoginGuardedButton;
use crate::user::UserState;

/// Renders a page requesting a login
#[component]
pub fn LoginWindow() -> impl IntoView {
    let user_state = expect_context::<UserState>();

    view! {
        <div class="hero">
            <div class="hero-content flex text-center">
                <AuthErrorIcon class="h-20 w-20 lg:h-44 lg:w-44"/>
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">"Not authenticated"</h1>
                    <p class="pt-4">"Sorry, we had some trouble identifying you."</p>
                    <p class="pb-4">"Please login to access this page."</p>
                    <ActionForm action=user_state.login_action>
                        <input type="text" name="redirect_url" class="hidden" value=use_location().pathname/>
                        <button type="submit" class="button-primary w-full">
                            {move_tr!("login")}
                        </button>
                    </ActionForm>
                </div>
            </div>
        </div>
    }
}

/// Component to display the author of a post or comment
#[component]
pub fn AuthorWidget(
    author: String,
    is_moderator: bool,
) -> impl IntoView {
    let navigate = use_navigate();
    let user_state = expect_context::<UserState>();
    let author_profile_path = get_profile_path(&author);
    let aria_label = format!("Navigate to user {}'s profile with path {}", author, author_profile_path);
    let author = StoredValue::new(author);

    view! {
        <button
            class="button-rounded-neutral px-2 py-1 flex gap-1.5 items-center"
            on:click=move |ev| {
                ev.prevent_default();
                navigate(author_profile_path.as_str(), NavigateOptions::default());
            }
            aria-label=aria_label
        >
            { move || if is_moderator {
                    view! { <ModeratorAuthorIcon/> }.into_any()
                } else {
                    view! {
                        <Transition fallback=move || view! { <LoadingIcon class="content-toolbar-icon-size"/> }>
                        {
                            move || Suspend::new(async move {
                                match &user_state.user.await {
                                    Ok(Some(user)) if author.with_value(|author| *author == user.username) => view! { <SelfAuthorIcon/> }.into_any(),
                                    _ => view! { <AuthorIcon/> }.into_any(),
                                }
                            })
                        }
                        </Transition>
                    }.into_any()
                }
            }
            <span class="text-xs xl:text-sm">{author.get_value()}</span>
        </button>
    }.into_any()
}

/// Component to display a button opening a modal dialog if the user
/// is authenticated and redirecting to a login page otherwise
#[component]
pub fn LoginGuardedOpenModalButton<IV>(
    show_dialog: RwSignal<bool>,
    #[prop(into)]
    button_class: Signal<&'static str>,
    children: TypedChildrenFn<IV>,
) -> impl IntoView
where
    IV: IntoView + 'static
{
    view! {
        <LoginGuardedButton
            button_class
            button_action=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
            children
            attr:aria-expanded=move || show_dialog.get().to_string()
            attr:aria-haspopup="dialog"
        />
    }
}

/// Component to render a delete button
#[component]
pub fn DeleteButton<A, O>(
    #[prop(into)]
    title: Signal<String>,
    id: i64,
    id_name: &'static str,
    author_id: i64,
    delete_action: ServerAction<A>
) -> impl IntoView
where
    A: DeserializeOwned
    + ServerFn<Protocol = Http<PostUrl, O>, Error = AppError>
    + Clone
    + Send
    + Sync
    + 'static,
    <<A::Client as Client<A::Error>>::Request as ClientReq<
        A::Error,
    >>::FormData: From<FormData>,
    A: Send + Sync + 'static,
    A::Output: Send + Sync + 'static,
    <A as ServerFn>::Client: Client<AppError>,
    O: 'static,
{
    let user_state = expect_context::<UserState>();
    let show_form = RwSignal::new(false);
    let show_button = move || match &(*user_state.user.read()) {
        Some(Ok(Some(user))) => user.user_id == author_id,
        _ => false,
    };
    let delete_button_class = move || match show_form.get() {
        true => "button-rounded-error",
        false => "button-rounded-neutral",
    };
    view! {
        <Suspense>
            <Show when=show_button>
                <div>
                    <button
                        class=delete_button_class
                        aria-expanded=move || show_form.get().to_string()
                        aria-haspopup="dialog"
                        on:click=move |_| show_form.update(|show: &mut bool| *show = !*show)
                    >
                        <DeleteIcon/>
                    </button>
                    <ModalDialog
                        class="w-full flex justify-center"
                        show_dialog=show_form
                    >
                        <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-5 w-96">
                            <div class="text-center font-bold text-2xl">{title}</div>
                            <div class="text-center font-bold text-xl">{move_tr!("delete-warning")}</div>
                            <ActionForm action=delete_action>
                                <input
                                    name=id_name
                                    class="hidden"
                                    value=id
                                />
                                <ModalFormButtons
                                    disable_publish=false
                                    show_form
                                />
                            </ActionForm>
                            <ActionError action=delete_action.into()/>
                        </div>
                    </ModalDialog>
                </div>
            </Show>
        </Suspense>
    }
}

/// Component to display a checkbox to enable or disable NSFW results.
/// If the user is not logged in or has disabled NSFW in his settings, the checkbox is hidden and deactivated.
#[component]
pub fn NsfwCheckbox(
    show_nsfw: RwSignal<bool>,
    #[prop(default = "NSFW")]
    label: &'static str,
    #[prop(default = "pl-1")]
    class: &'static str,
) -> impl IntoView {
    let user_state = expect_context::<UserState>();
    view! {
        <Transition fallback=move || view! {  <LoadingIcon/> }>
        {
            move || Suspend::new(async move {
                match user_state.user.await {
                    Ok(Some(user)) if user.show_nsfw => Some(view! {
                        <LabeledSignalCheckbox label value=show_nsfw class=class/>
                    }),
                    _ => {
                        show_nsfw.set(false);
                        None
                    },
                }
            })
        }
        </Transition>
    }
}