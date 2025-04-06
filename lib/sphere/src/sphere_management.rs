use chrono::SecondsFormat;
use leptos::ev::{Event, SubmitEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{FileReader, FormData, HtmlFormElement, HtmlInputElement};
use leptos_router::components::Outlet;
use leptos_use::{signal_debounced, use_textarea_autosize};
use strum::IntoEnumIterator;


use sharesphere_utils::editor::{FormTextEditor, TextareaData};
use sharesphere_utils::errors::{AppError, ErrorDisplay};
use sharesphere_utils::icons::{CrossIcon, LoadingIcon, MagnifierIcon, SaveIcon};
use sharesphere_utils::unpack::{SuspenseUnpack, TransitionUnpack};
use sharesphere_utils::widget::{EnumDropdown, ModalDialog, IMAGE_FILE_PARAM, SPHERE_NAME_PARAM};

use sharesphere_auth::role::{AuthorizedShow, PermissionLevel, SetUserSphereRole};

use sharesphere_core::sphere::{Sphere};

use crate::rule::SphereRulesPanel;
use crate::satellite::SatellitePanel;
use crate::sphere_category::SphereCategoriesDialog;

use sharesphere_auth::auth_widget::LoginWindow;
use sharesphere_core::moderation::{get_moderation_info, ModerationInfoDialog};
use sharesphere_core::search::get_matching_user_header_vec;
use sharesphere_core::sphere_management::{get_sphere_ban_vec, set_sphere_banner, set_sphere_icon, RemoveUserBan};
use sharesphere_core::state::{GlobalState, SphereState};

pub const MANAGE_SPHERE_ROUTE: &str = "/manage";
pub const NONE_STR: &str = "None";
pub const DAY_STR: &str = "day";
pub const DAYS_STR: &str = "days";
pub const PERMANENT_STR: &str = "Permanent";
pub const MISSING_SPHERE_STR: &str = "Missing sphere name.";
pub const MISSING_BANNER_FILE_STR: &str = "Missing banner file.";


/// Component to guard the sphere cockpit
#[component]
pub fn SphereCockpitGuard() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_name = expect_context::<SphereState>().sphere_name;
    view! {
        <SuspenseUnpack resource=state.user let:user>
        {
            match user {
                Some(user) => {
                    match user.check_permissions(&sphere_name.read_untracked(), PermissionLevel::Moderate) {
                        Ok(_) => view! { <Outlet/> }.into_any(),
                        Err(error) => view! { <ErrorDisplay error/> }.into_any(),
                    }
                },
                None => view! { <LoginWindow/> }.into_any(),
            }
        }
        </SuspenseUnpack>
    }.into_any()
}

/// Component to manage a sphere
#[component]
pub fn SphereCockpit() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-5 overflow-y-auto w-full 2xl:w-2/3 mx-auto">
            <div class="text-2xl text-center">"Sphere Cockpit"</div>
            <SphereDescriptionDialog/>
            <SphereIconDialog/>
            <SphereBannerDialog/>
            <SatellitePanel/>
            <SphereCategoriesDialog/>
            <ModeratorPanel/>
            <SphereRulesPanel/>
            <BanPanel/>
        </div>
    }.into_any()
}

/// Component to edit a sphere's description
#[component]
pub fn SphereDescriptionDialog() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = expect_context::<SphereState>().sphere_name;
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 items-center w-full h-fit bg-base-200 p-2 rounded-sm">
                <div class="text-xl text-center">"Sphere description"</div>
                <SuspenseUnpack resource=sphere_state.sphere_resource let:sphere>
                    <SphereDescriptionForm sphere=sphere/>
                </SuspenseUnpack>
            </div>
        </AuthorizedShow>
    }
}

/// Form to edit a sphere's description
#[component]
pub fn SphereDescriptionForm<'a>(
    sphere: &'a Sphere,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let description_autosize = use_textarea_autosize(textarea_ref);
    let description_data = TextareaData {
        content: description_autosize.content,
        set_content: description_autosize.set_content,
        textarea_ref
    };
    description_data.set_content.set(sphere.description.clone());
    let disable_submit = move || description_data.content.read().is_empty();
    view! {
        <ActionForm
            action=sphere_state.update_sphere_desc_action
            attr:class="flex flex-col gap-1"
        >
            <input
                name="sphere_name"
                class="hidden"
                value=sphere_state.sphere_name
            />
            <FormTextEditor
                name="description"
                placeholder="Description"
                data=description_data
            />
            <button
                type="submit"
                class="button-secondary self-end"
                disabled=disable_submit
            >
                <SaveIcon/>
            </button>
        </ActionForm>
    }
}

/// Component to edit a sphere's icon
#[component]
pub fn SphereIconDialog() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    let set_icon_action = Action::new_local(|data: &FormData| {
        set_sphere_icon(data.clone().into())
    });
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 items-center w-full h-fit bg-base-200 p-2 rounded-sm">
                <div class="text-xl text-center">"Sphere icon"</div>
                <SphereImageForm
                    sphere_name=sphere_state.sphere_name
                    action=set_icon_action
                    preview_class="max-h-12 max-w-full object-contain"
                />
            </div>
        </AuthorizedShow>
    }
}

/// Component to edit a sphere's banner
#[component]
pub fn SphereBannerDialog() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    let set_banner_action = Action::new_local(|data: &FormData| {
        set_sphere_banner(data.clone().into())
    });
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 items-center w-full h-fit bg-base-200 p-2 rounded-sm">
                <div class="text-xl text-center">"Sphere banner"</div>
                <SphereImageForm
                    sphere_name=sphere_state.sphere_name
                    action=set_banner_action
                />
            </div>
        </AuthorizedShow>
    }
}

/// Form to upload an image to the server
/// The form contains two inputs: a hidden sphere name and an image form
#[component]
pub fn SphereImageForm(
    #[prop(into)]
    sphere_name: Signal<String>,
    action: Action<FormData, Result<(), ServerFnError<AppError>>, LocalStorage>,
    #[prop(default = "max-h-80 max-w-full object-contain")]
    preview_class: &'static str,
) -> impl IntoView {
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
        let form_data = FormData::new_with_form(&target).unwrap();
        action.dispatch_local(form_data);
    };

    let preview_url = RwSignal::new(String::new());
    let on_file_change = move |ev| {
        let input: HtmlInputElement = event_target::<HtmlInputElement>(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                // Try to create a FileReader, returning early if it fails
                let reader = match FileReader::new() {
                    Ok(reader) => reader,
                    Err(_) => {
                        log::error!("Failed to create file reader.");
                        return
                    }, // Return early if FileReader creation fails
                };

                // Set up the onload callback for FileReader
                let preview_url_clone = preview_url.clone();
                let onload_callback = Closure::wrap(Box::new(move |e: Event| {
                    if let Some(reader) = e.target().and_then(|t| t.dyn_into::<FileReader>().ok()) {
                        if let Ok(Some(result)) = reader.result().and_then(|r| Ok(r.as_string())) {
                            preview_url_clone.set(result); // Update the preview URL
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                reader.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
                onload_callback.forget(); // Prevent the closure from being dropped

                // Start reading the file as a Data URL, returning early if it fails
                if let Err(e) = reader.read_as_data_url(&file) {
                    let error_message = e.as_string().unwrap_or_else(|| format!("{:?}", e));
                    log::error!("Error while getting preview of local image: {error_message}");
                };
            }
        }
    };

    view! {
        <form on:submit=on_submit class="flex flex-col gap-1">
            <input
                name=SPHERE_NAME_PARAM
                class="hidden"
                value=sphere_name
            />
            <input
                type="file"
                name=IMAGE_FILE_PARAM
                accept="image/*"
                class="file-input file-input-primary w-full rounded-xs"
                on:change=on_file_change
            />
            <Show when=move || !preview_url.read().is_empty()>
                <img src=preview_url alt="Image Preview" class=preview_class/>
            </Show>
            <button
                type="submit"
                class="button-secondary self-end"
            >
                <SaveIcon/>
            </button>
            {move || {
                if action.pending().get()
                {
                    view! { <LoadingIcon/> }.into_any()
                } else {
                    match action.value().get()
                    {
                        Some(Ok(())) => {
                            if let Some(state) = use_context::<GlobalState>() {
                                state.sphere_reload_signal.update(|value| *value += 1);
                            }
                            ().into_any()
                        }
                        Some(Err(e)) => view! { <ErrorDisplay error=e.into()/> }.into_any(),
                        None => ().into_any()
                    }
                }
            }}
        </form>
    }
}

/// Component to manage moderators
#[component]
pub fn ModeratorPanel() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    let username_input = RwSignal::new(String::default());
    let select_ref = NodeRef::<html::Select>::new();

    let set_role_action = ServerAction::<SetUserSphereRole>::new();

    view! {
        // TODO add overflow-y-auto max-h-full?
        <div class="shrink-0 flex flex-col gap-1 items-center w-full h-fit bg-base-200 p-2 rounded-sm">
            <div class="text-xl text-center">"Moderators"</div>
            <div class="flex flex-col gap-1">
                <div class="flex gap-1 border-b border-base-content/20">
                    <div class="w-2/5 px-4 py-2 text-left font-bold">Username</div>
                    <div class="w-2/5 px-4 py-2 text-left font-bold">Role</div>
                </div>
                <TransitionUnpack resource=sphere_state.sphere_roles_resource let:sphere_role_vec>
                {
                    sphere_role_vec.iter().enumerate().map(|(index, role)| {
                        let username = role.username.clone();
                        view! {
                            <div
                                class="flex gap-1 py-1 rounded-sm hover:bg-base-200 active:scale-95 transition duration-250"
                                on:click=move |_| {
                                    username_input.set(username.clone());
                                    match select_ref.get_untracked() {
                                        Some(select_ref) => select_ref.set_selected_index(index as i32),
                                        None => log::error!("Form permission level select failed to load."),
                                    };
                                }
                            >
                                <div class="w-2/5 px-4 select-none">{role.username.clone()}</div>
                                <div class="w-2/5 px-4 select-none">{role.permission_level.to_string()}</div>
                            </div>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
            <PermissionLevelForm
                sphere_name
                username_input
                select_ref
                set_role_action
            />
        </div>
    }
}

/// Component to set permission levels for a sphere
#[component]
pub fn PermissionLevelForm(
    sphere_name: Memo<String>,
    username_input: RwSignal<String>,
    select_ref: NodeRef<html::Select>,
    set_role_action: ServerAction<SetUserSphereRole>
) -> impl IntoView {
    let username_debounced: Signal<String> = signal_debounced(username_input, 250.0);
    let matching_user_resource = Resource::new(
        move || username_debounced.get(),
        move |username| async {
            if username.is_empty() {
                Ok(Vec::new())
            } else {
                get_matching_user_header_vec(username, Some(true), 20).await
            }
        },
    );
    let disable_submit = move || username_input.read().is_empty();

    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            <ActionForm action=set_role_action>
                <input
                    name="sphere_name"
                    class="hidden"
                    value=sphere_name
                />
                <div class="flex gap-1 items-center">
                    <div class="dropdown dropdown-end w-2/5">
                        <input
                            tabindex="0"
                            type="text"
                            name="username"
                            placeholder="Username"
                            autocomplete="off"
                            class="input input-primary w-full"
                            on:input=move |ev| {
                                username_input.set(event_target_value(&ev).to_lowercase());
                            }
                            prop:value=username_input
                        />
                        <Show when=move || !username_input.read().is_empty()>
                            <TransitionUnpack resource=matching_user_resource let:user_header_vec>
                            {
                                let user_header_vec = user_header_vec.clone();
                                view ! {
                                    <ul tabindex="0" class="dropdown-content z-1 menu p-2 shadow-sm bg-base-200 rounded-box w-2/5">
                                        <For
                                            each=move || user_header_vec.clone().into_iter()
                                            key=|user_header| user_header.username.clone()
                                            let(user_header)
                                        >
                                            <li>
                                                <button
                                                    type="button"
                                                    value=user_header.username
                                                    on:click=move |ev| username_input.set(event_target_value(&ev))
                                                >
                                                    {user_header.username.clone()}
                                                </button>
                                            </li>
                                        </For>
                                    </ul>
                                }
                            }
                            </TransitionUnpack>
                        </Show>
                    </div>
                    <EnumDropdown
                        name="permission_level"
                        enum_iter=PermissionLevel::iter()
                        select_ref
                    />
                    <button
                        type="submit"
                        class="button-secondary"
                        disabled=disable_submit
                    >
                        "Assign"
                    </button>
                </div>
            </ActionForm>
        </AuthorizedShow>
    }
}

/// Component to manage ban users
#[component]
pub fn BanPanel() -> impl IntoView {
    let sphere_name = expect_context::<SphereState>().sphere_name;
    let username_input = RwSignal::new(String::default());
    let username_debounced: Signal<String> = signal_debounced(username_input, 250.0);

    let unban_action = ServerAction::<RemoveUserBan>::new();
    let banned_users_resource = Resource::new(
        move || (username_debounced.get(), unban_action.version().get()),
        move |(username, _)| get_sphere_ban_vec(sphere_name.get_untracked(), username)
    );

    view! {
        // TODO add overflow-y-auto max-h-full?
        <div class="shrink-0 flex flex-col gap-1 items-center w-full bg-base-200 p-2 rounded-sm">
            <div class="text-xl text-center">"Banned users"</div>
            <div class="flex flex-col gap-1">
                <div class="flex flex-col border-b border-base-content/20">
                    <div class="flex">
                        <input
                            class="input input-primary px-6 w-2/5"
                            placeholder="Username"
                            value=username_input
                            on:input=move |ev| username_input.set(event_target_value(&ev))
                        />
                        <div class="w-2/5 px-6 py-2 text-left font-bold">Until</div>
                    </div>
                </div>
                <TransitionUnpack resource=banned_users_resource let:banned_user_vec>
                {
                    banned_user_vec.iter().map(|user_ban| {
                        let duration_string = match user_ban.until_timestamp {
                            Some(until_timestamp) => until_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
                            None => String::from("Permanent"),
                        };
                        let ban_id = user_ban.ban_id;
                        view! {
                            <div class="flex">
                                <div class="w-2/5 px-6">{user_ban.username.clone()}</div>
                                <div class="w-2/5 px-6">{duration_string}</div>
                                <div class="w-1/5 flex justify-end gap-1">
                                    <BanInfoButton
                                        post_id=user_ban.post_id
                                        comment_id=user_ban.comment_id
                                    />
                                    <AuthorizedShow sphere_name permission_level=PermissionLevel::Ban>
                                        <ActionForm action=unban_action>
                                            <input
                                                name="ban_id"
                                                class="hidden"
                                                value=ban_id
                                            />
                                            <button class="p-1 h-full rounded-xs bg-error hover:bg-error/75 active:scale-90 transition duration-250">
                                                <CrossIcon/>
                                            </button>
                                        </ActionForm>
                                    </AuthorizedShow>
                                </div>
                            </div>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
        </div>
    }
}

/// Component to display a button opening a modal dialog with a ban's details
#[component]
pub fn BanInfoButton(
    post_id: i64,
    comment_id: Option<i64>,
) -> impl IntoView {
    let show_dialog = RwSignal::new(false);

    view! {
        <button
            class="p-1 h-full bg-secondary rounded-xs hover:bg-secondary/75 active:scale-90 transition duration-250"
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            <MagnifierIcon/>
        </button>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            {
                let ban_detail_resource = Resource::new(
                    move || (),
                    move |_| get_moderation_info(post_id, comment_id)
                );
                view! {
                    <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
                        <SuspenseUnpack resource=ban_detail_resource let:moderation_info>
                            <ModerationInfoDialog moderation_info/>
                            <button
                                type="button"
                                class="p-1 h-full rounded-xs bg-error hover:bg-error/75 active:scale-95 transition duration-250"
                                on:click=move |_| show_dialog.set(false)
                            >
                                "Close"
                            </button>
                        </SuspenseUnpack>
                    </div>
                }
            }
        </ModalDialog>
    }
}