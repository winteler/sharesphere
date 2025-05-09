use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

use sharesphere_utils::form::LabeledFormCheckbox;
use sharesphere_utils::icons::{LoadingIcon, UserIcon, UserSettingsIcon};
use sharesphere_utils::routes::{get_profile_path, get_username_memo};
use sharesphere_utils::unpack::{handle_additional_load, handle_initial_load, reset_additional_load, ActionError};
use sharesphere_utils::widget::{EnumQueryTabs, ModalDialog, ModalFormButtons, ToView};

use sharesphere_auth::auth::NavigateToUserAccount;
use sharesphere_auth::user::{UserHeader, UserHeaderWidget};

use sharesphere_core::comment::{CommentMiniatureList};
use sharesphere_core::post::{PostListWithInitLoad, POST_BATCH_SIZE};
use sharesphere_core::profile::{get_user_comment_vec, get_user_post_vec};
use sharesphere_core::ranking::{CommentSortType, CommentSortWidget, PostSortType, PostSortWidget, SortType};
use sharesphere_core::sidebar::HomeSidebar;
use sharesphere_core::state::GlobalState;

pub const PROFILE_TAB_QUERY_PARAM: &str = "tab";

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ProfileTabs {
    #[default]
    Posts,
    Comments,
}

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SelfProfileTabs {
    #[default]
    Posts,
    Comments,
    Settings,
}

impl ToView for ProfileTabs {
    fn to_view(self) -> impl IntoView + 'static {
        match self {
            ProfileTabs::Posts => view! { <UserPosts/> }.into_any(),
            ProfileTabs::Comments => view! { <UserComments/> }.into_any(),
        }
    }
}

impl ToView for SelfProfileTabs {
    fn to_view(self) -> impl IntoView + 'static {
        match self {
            SelfProfileTabs::Posts => view! { <UserPosts/> }.into_any(),
            SelfProfileTabs::Comments => view! { <UserComments/> }.into_any(),
            SelfProfileTabs::Settings => view! { <UserSettings/> }.into_any(),
        }
    }
}

/// Displays a user's profile
#[component]
pub fn UserProfile() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let params = use_params_map();
    let query_username = get_username_memo(params);
    view! {
        <div class="w-full flex justify-center">
            <div class="w-full 2xl:w-2/3 flex flex-col max-2xl:items-center">
                <div class="p-2 pt-4 flex items-center gap-1 text-2xl font-bold">
                    <UserIcon/>
                    {move || query_username.get()}
                </div>
                <Transition fallback=move || view! {  <LoadingIcon/> }>
                { 
                    move || Suspend::new(async move { 
                        match state.user.await {
                            Ok(Some(user)) if user.username == query_username.get() => view! { 
                                <EnumQueryTabs 
                                    query_param=PROFILE_TAB_QUERY_PARAM 
                                    query_enum_iter=SelfProfileTabs::iter()
                                /> 
                            }.into_any(),
                            _ => view! { 
                                <EnumQueryTabs 
                                    query_param=PROFILE_TAB_QUERY_PARAM 
                                    query_enum_iter=ProfileTabs::iter()
                                /> 
                            }.into_any(),
                        }
                    })
                }
                </Transition>
            </div>
        </div>
        <div class="max-2xl:hidden">
            <HomeSidebar/>
        </div>
    }
}

/// Displays a user's posts
#[component]
pub fn UserPosts() -> impl IntoView {
    let params = use_params_map();
    let username = get_username_memo(params);
    let sort_signal = RwSignal::new(SortType::Post(PostSortType::Hot));
    let additional_post_vec = RwSignal::new(Vec::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let post_vec_resource = Resource::new(
        move || (username.get(), sort_signal.get()),
        move |(username, sort_type)| async move {
            #[cfg(feature = "hydrate")]
            is_loading.set(true);
            reset_additional_load(additional_post_vec, additional_load_count, Some(list_ref));
            let result = get_user_post_vec(username, sort_type, 0).await;
            #[cfg(feature = "hydrate")]
            is_loading.set(false);
            result
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let num_post = (POST_BATCH_SIZE as usize) + additional_post_vec.read_untracked().len();
                let additional_load = get_user_post_vec(username.get_untracked(), sort_signal.get_untracked(), num_post).await;
                handle_additional_load(additional_load, additional_post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <PostSortWidget sort_signal/>
        <PostListWithInitLoad
            post_vec_resource
            additional_post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}

/// Displays a user's comments
#[component]
pub fn UserComments() -> impl IntoView {
    let params = use_params_map();
    let username = get_username_memo(params);
    let sort_signal = RwSignal::new(SortType::Comment(CommentSortType::Recent));
    let comment_vec = RwSignal::new(Vec::new());
    let additional_load_count = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_comment_resource = LocalResource::new(
        move || async move {
            is_loading.set(true);
            let initial_load = get_user_comment_vec(username.get(), sort_signal.get(), 0).await;
            handle_initial_load(initial_load, comment_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_comment_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let additional_load = get_user_comment_vec(
                    username.get_untracked(),
                    sort_signal.get_untracked(),
                    comment_vec.read_untracked().len(),
                ).await;
                handle_additional_load(additional_load, comment_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <CommentSortWidget sort_signal/>
        <CommentMiniatureList
            comment_vec
            is_loading
            load_error
            additional_load_count
            list_ref
        />
    }
}

/// Displays a user's settings
#[component]
pub fn UserSettings() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    view! {
        <div class="self-center flex flex-col gap-3 w-3/4 2xl:w-1/2">
            <Suspense fallback=move || view! {  <LoadingIcon/> }>
            {
                move || Suspend::new(async move {
                    let (is_nsfw, show_nsfw, days_hide_spoiler) = match state.user.await {
                        Ok(Some(user)) => (user.is_nsfw, user.show_nsfw, user.days_hide_spoiler.unwrap_or_default()),
                        _ => (false, false, 0),
                    };
                    view! {
                        <ActionForm action=state.set_settings_action attr:class="flex flex-col gap-3">
                            <LabeledFormCheckbox name="is_nsfw" label="NSFW profile" value=is_nsfw/>
                            <LabeledFormCheckbox name="show_nsfw" label="Show NSFW" value=show_nsfw/>
                            <div class="flex justify-between items-center">
                                "Hide spoilers duration (days)"
                                <input
                                    type="number"
                                    min="0"
                                    max="999"
                                    name="days_hide_spoilers"
                                    class="input input-primary no-spinner text-right w-16"
                                    autocomplete="off"
                                    value=days_hide_spoiler
                                />
                            </div>
                            <button type="submit" class="button-secondary">
                                "Save"
                            </button>
                        </ActionForm>
                        <ActionError action=state.set_settings_action.into()/>
                    }
                })
            }
            </Suspense>
            <div class="flex justify-between items-center">
                <UserAccountButton/>
                <DeleteUserButton/>
            </div>
        </div>
    }
}

/// Button to delete one's account
#[component]
pub fn DeleteUserButton() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let show_dialog = RwSignal::new(false);
    view! {
        <button
            class="button-error"
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            "Delete your account"
        </button>
        <ModalDialog
            class="w-full max-w-lg"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Delete your account"</div>
                <div class="text-center font-bold text-xl">"This cannot be undone."</div>
                <ActionForm action=state.delete_user_action>
                    <ModalFormButtons
                        disable_publish=false
                        show_form=show_dialog
                    />
                </ActionForm>
                <ActionError action=state.delete_user_action.into()/>
            </div>
        </ModalDialog>
    }
}

/// Button to navigate to the user's account on the OIDC provider
#[component]
pub fn UserAccountButton() -> impl IntoView {
    let navigate_to_account_action = ServerAction::<NavigateToUserAccount>::new();
    view! {
        <ActionForm action=navigate_to_account_action attr:class="flex justify-center items-center">
            <button type="submit" class="button-primary flex">
                <UserSettingsIcon/>
                "Account"
            </button>
        </ActionForm>
    }
}

/// Component to display a user header and redirect to his profile upon click
#[component]
pub fn UserHeaderLink<'a>(
    user_header: &'a UserHeader,
) -> impl IntoView {
    let user_profile_path = get_profile_path(&user_header.username);
    view! {
        <a href=user_profile_path class="w-full h-fit p-2 rounded-sm hover:bg-base-200">
            <UserHeaderWidget user_header/>
        </a>
    }.into_any()
}
