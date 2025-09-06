use std::collections::HashMap;
use leptos::html;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_params_map;
use leptos_use::{signal_throttled_with_options, ThrottleOptions};
use url::Url;
use sharesphere_utils::editor::{FormMarkdownEditor, FormTextEditor, TextareaData};
use sharesphere_utils::embed::EmbedType;
use sharesphere_utils::form::LabeledFormCheckbox;
use sharesphere_utils::icons::{CrossIcon, EditIcon, LinkIcon, PauseIcon, PlayIcon, PlusIcon};
use sharesphere_utils::routes::{get_satellite_id_memo, get_satellite_path};
use sharesphere_utils::unpack::{handle_additional_load, reset_additional_load, ActionError, SuspenseUnpack, TransitionUnpack};
use sharesphere_utils::widget::{ContentBody, ModalDialog, ModalFormButtons, TagsWidget};

use sharesphere_auth::role::{AuthorizedShow, PermissionLevel};

use sharesphere_core::post::{add_sphere_info_to_post_vec, get_post_vec_by_satellite_id, CreatePost, PostForm, PostListWithInitLoad, PostWithSphereInfo, POST_BATCH_SIZE};
use sharesphere_core::ranking::{PostSortType, SortType};
use sharesphere_core::satellite::{get_satellite_by_id, get_satellite_vec_by_sphere_name, Satellite, SatelliteState};
use sharesphere_core::sphere::get_sphere_with_user_info;
use sharesphere_core::sphere_category::get_sphere_category_vec;
use sharesphere_core::state::SphereState;
use sharesphere_utils::checks::{check_satellite_name, check_string_length};
use sharesphere_utils::constants::{MAX_CONTENT_LENGTH, MAX_SATELLITE_NAME_LENGTH, SCROLL_LOAD_THROTTLE_DELAY};
use crate::sphere::{SphereToolbar};
use crate::sphere_category::{get_sphere_category_header_map};

/// Component to display a satellite banner
#[component]
pub fn SatelliteBanner() -> impl IntoView {
    let params = use_params_map();
    let satellite_id = get_satellite_id_memo(params);
    let satellite_state = SatelliteState {
        satellite_id,
        sort_type: RwSignal::new(SortType::Post(PostSortType::Hot)),
        category_id_filter: RwSignal::new(None),
        satellite_resource: Resource::new(
            move || satellite_id.get(),
            move |satellite_id| get_satellite_by_id(satellite_id)
        ),
    };
    provide_context(satellite_state);

    view! {
        <TransitionUnpack resource=satellite_state.satellite_resource let:satellite>
            <div class="w-1/2 lg:w-1/4">
                <SatelliteHeader
                    satellite_name=satellite.satellite_name.clone()
                    satellite_link=get_satellite_path(&satellite.sphere_name, satellite.satellite_id)
                    is_spoiler=satellite.is_spoiler
                    is_nsfw=satellite.is_nsfw
                />
            </div>
        </TransitionUnpack>
        <Outlet/>
    }
}

/// Component to a satellite's header
#[component]
pub fn SatelliteHeader(
    satellite_name: String,
    satellite_link: String,
    is_spoiler: bool,
    is_nsfw: bool,
) -> impl IntoView {
    view! {
        <a
            href=satellite_link
            class="p-2 border border-1 border-base-content/20 rounded-sm hover:bg-base-200 flex flex-col gap-1"
        >
            {satellite_name}
            <TagsWidget is_spoiler=is_spoiler is_nsfw=is_nsfw/>
        </a>
    }
}

/// Component to display a satellite's content
#[component]
pub fn SatelliteContent() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let satellite_state = expect_context::<SatelliteState>();

    let sphere_with_sub_resource = Resource::new(
        move || (sphere_state.sphere_name.get(),),
        move |(sphere_name,)| get_sphere_with_user_info(sphere_name),
    );

    let category_id_signal = RwSignal::new(None);
    let sort_signal = RwSignal::new(SortType::Post(PostSortType::Hot));
    let additional_load_count = RwSignal::new(0);
    let additional_post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let is_category_map_loaded = RwSignal::new(false);
    let sphere_category_header_map = RwSignal::new(HashMap::new());

    let post_vec_resource = Resource::new(
        move || (
            satellite_state.satellite_id.get(),
            category_id_signal.get(),
            sort_signal.get(),
            sphere_state.post_refresh_count.get(),
        ),
        move |(satellite_id, category_id, sort_type, _)| async move {
            #[cfg(feature = "hydrate")]
            is_loading.set(true);
            reset_additional_load(additional_post_vec, additional_load_count, Some(list_ref));

            sphere_category_header_map.set(get_sphere_category_header_map(sphere_state.sphere_categories_resource.clone().await));
            is_category_map_loaded.set(true);

            let result = get_post_vec_by_satellite_id(
                satellite_id,
                category_id,
                sort_type,
                0
            ).await.map(|post_vec| add_sphere_info_to_post_vec(post_vec, &*sphere_category_header_map.read_untracked(), None));
            #[cfg(feature = "hydrate")]
            is_loading.set(false);
            result
        }
    );

    let additional_load_count_throttled: Signal<i32> = signal_throttled_with_options(
        additional_load_count,
        SCROLL_LOAD_THROTTLE_DELAY,
        ThrottleOptions::default().leading(true).trailing(false)
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count_throttled.get() > 0 {
                is_loading.set(true);
                if !is_category_map_loaded.get_untracked() {
                    sphere_category_header_map.set(get_sphere_category_header_map(sphere_state.sphere_categories_resource.clone().await));
                    is_category_map_loaded.set(true);
                }
                let num_post = (POST_BATCH_SIZE as usize) + additional_post_vec.read_untracked().len();
                let additional_load = get_post_vec_by_satellite_id(
                    satellite_state.satellite_id.get_untracked(),
                    category_id_signal.get_untracked(),
                    sort_signal.get_untracked(),
                    num_post
                ).await.map(|post_vec| add_sphere_info_to_post_vec(post_vec, &*sphere_category_header_map.read_untracked(), None));
                handle_additional_load(additional_load, additional_post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <TransitionUnpack resource=satellite_state.satellite_resource let:satellite>
            <div class="p-2">
                <ContentBody
                    body=satellite.body.clone()
                    is_markdown=satellite.markdown_body.is_some()
                />
            </div>
        </TransitionUnpack>
        <SuspenseUnpack resource=sphere_with_sub_resource let:sphere>
            <SphereToolbar
                sphere
                sort_signal
            />
        </SuspenseUnpack>
        <PostListWithInitLoad
            post_vec_resource
            additional_post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
            show_sphere_header=false
        />
    }
}

/// Component to create a post in a satellite
#[component]
pub fn CreateSatellitePost() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let satellite_state = expect_context::<SatelliteState>();

    let create_post_action = ServerAction::<CreatePost>::new();

    let title_input = RwSignal::new(String::default());
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_data = TextareaData {
        content: RwSignal::new(String::default()),
        textarea_ref,
    };
    let link_input = RwSignal::new(String::default());
    let embed_type_input = RwSignal::new(EmbedType::None);

    let category_vec_resource = Resource::new(
        move || sphere_state.sphere_name.get(),
        move |sphere_name| get_sphere_category_vec(sphere_name)
    );

    view! {
        <div class="w-full lg:w-2/5 p-2 mx-auto flex flex-col gap-2 overflow-auto">
            <ActionForm action=create_post_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Share a post!"</h2>
                    <input
                        type="text"
                        name="post_location[sphere]"
                        class="hidden"
                        value=sphere_state.sphere_name
                    />
                    <input
                        type="text"
                        name="post_location[satellite_id]"
                        class="hidden"
                        value=satellite_state.satellite_id
                    />
                    <SuspenseUnpack resource=satellite_state.satellite_resource let:satellite>
                        <PostForm
                            title_input
                            body_data
                            embed_type_input
                            link_input
                            sphere_name=sphere_state.sphere_name
                            is_parent_spoiler=satellite.is_spoiler
                            is_parent_nsfw=satellite.is_nsfw
                            category_vec_resource
                        />
                    </SuspenseUnpack>
                    <button type="submit" class="button-secondary" disabled=move || {
                        title_input.read().is_empty() ||
                        (
                            body_data.content.read().is_empty() &&
                            *embed_type_input.read() == EmbedType::None
                        ) || (
                            *embed_type_input.read() != EmbedType::None &&
                            link_input.with(|link| link.is_empty() || Url::parse(link).is_err())
                        )
                    }>
                        "Submit"
                    </button>
                </div>
            </ActionForm>
            <ActionError action=create_post_action.into()/>
        </div>
    }
}

/// Component to display active satellites for the current sphere
#[component]
pub fn ActiveSatelliteList() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();

    view! {
        <TransitionUnpack resource=sphere_state.satellite_vec_resource let:satellite_vec>
        {
            match satellite_vec.is_empty() {
                true => None,
                false => {
                    let satellite_list = satellite_vec.iter().map(|satellite| {
                        let satellite_name = satellite.satellite_name.clone();
                        let satellite_link = get_satellite_path(&satellite.sphere_name, satellite.satellite_id);
                        view! {
                            <SatelliteHeader
                                satellite_name
                                satellite_link
                                is_spoiler=satellite.is_spoiler
                                is_nsfw=satellite.is_nsfw
                            />
                        }
                    }).collect_view();

                    Some(view! {
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-2">
                            {satellite_list}
                        </div>
                    })
                },
            }
        }
        </TransitionUnpack>
    }
}

/// Component to manage satellites
#[component]
pub fn SatellitePanel() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let satellite_vec_resource = Resource::new(
        move || (
            sphere_state.sphere_name.get(),
            sphere_state.create_satellite_action.version().get(),
            sphere_state.update_satellite_action.version().get(),
            sphere_state.disable_satellite_action.version().get(),
        ),
        move |(sphere_name, _, _, _)| get_satellite_vec_by_sphere_name(sphere_name, true)
    );
    view! {
        // TODO add overflow-y-auto max-h-full?
        <div class="flex flex-col gap-1 items-center w-full h-fit bg-base-200 p-2 rounded-sm">
            <div class="text-xl text-center">"Satellites"</div>
            <div class="w-full flex flex-col gap-1">
                <div class="border-b border-base-content/20 pl-1">
                    <div class="w-5/6 flex gap-1">
                        <div class="w-3/6 py-2 font-bold">"Title"</div>
                        <div class="w-20 py-2 font-bold text-center">"Active"</div>
                        <div class="w-20 py-2 font-bold text-center">"Link"</div>
                    </div>
                </div>
                <TransitionUnpack resource=satellite_vec_resource let:satellite_vec>
                {
                    satellite_vec.iter().map(|satellite| {
                        let show_edit_form = RwSignal::new(false);
                        let satellite_name = satellite.satellite_name.clone();
                        let satellite_link = get_satellite_path(&satellite.sphere_name, satellite.satellite_id);
                        let satellite = satellite.clone();
                        view! {
                            <div class="flex gap-1 justify-between rounded-sm pl-1">
                                <div class="w-5/6 flex gap-1">
                                    <div class="w-3/6 select-none">{satellite_name}</div>
                                    <div class="w-20 flex justify-center items-center">
                                    {
                                        match satellite.disable_timestamp.is_none() {
                                            true => view! { <PlayIcon/> }.into_any(),
                                            false => view! { <PauseIcon/> }.into_any(),
                                        }
                                    }
                                    </div>
                                    <div class="w-20 flex justify-center items-center">
                                        <a href=satellite_link class="button-rounded-ghost">
                                            <LinkIcon/>
                                        </a>
                                    </div>
                                </div>
                                <div class="flex gap-1 justify-end">
                                    <button
                                        class="button-secondary"
                                        on:click=move |_| show_edit_form.update(|value| *value = !*value)
                                    >
                                        <EditIcon/>
                                    </button>
                                    <DisableSatelliteButton satellite_id=satellite.satellite_id/>
                                </div>
                            </div>
                            <ModalDialog
                                class="w-full max-w-xl"
                                show_dialog=show_edit_form
                            >
                                <EditSatelliteForm satellite=satellite.clone() show_form=show_edit_form/>
                            </ModalDialog>
                        }
                    }).collect_view()
                }
                </TransitionUnpack>
            </div>
            <CreateSatelliteForm/>
        </div>
    }
}

/// Component to disable a satellite
#[component]
pub fn DisableSatelliteButton(
    satellite_id: i64
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            <ActionForm
                action=sphere_state.disable_satellite_action
                attr:class="h-fit flex justify-center"
            >
                <input
                    name="satellite_id"
                    class="hidden"
                    value=satellite_id
                />
                <button class="button-error">
                    <CrossIcon/>
                </button>
            </ActionForm>
        </AuthorizedShow>
    }
}

/// Component to edit a satellite
#[component]
pub fn EditSatelliteForm(
    satellite: Satellite,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let is_nsfw = satellite.is_nsfw;
    let is_spoiler = satellite.is_spoiler;
    let title_ref = NodeRef::<html::Textarea>::new();
    let title_data = TextareaData {
        content: RwSignal::new(satellite.satellite_name),
        textarea_ref: title_ref,
    };
    let body_ref = NodeRef::<html::Textarea>::new();
    let (body, is_markdown_body) = match satellite.markdown_body {
        Some(markdown_body) => (markdown_body, true),
        None => (satellite.body, false),
    };
    let body_data = TextareaData {
        content: RwSignal::new(body),
        textarea_ref: body_ref,
    };

    let invalid_inputs = are_satellite_inputs_invalid(title_data.content.into(), body_data.content.into());

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Edit a satellite"</div>
            <ActionForm action=sphere_state.update_satellite_action>
                <input
                    name="satellite_id"
                    class="hidden"
                    value=satellite.satellite_id
                />
                <div class="flex flex-col gap-3 w-full">
                    <SatelliteInputs title_data body_data is_markdown_body is_nsfw is_spoiler/>
                    <ModalFormButtons
                        disable_publish=invalid_inputs
                        show_form
                    />
                </div>
            </ActionForm>
        </div>
    }
}

/// Component to create a satellite
#[component]
pub fn CreateSatelliteForm() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let show_dialog = RwSignal::new(false);
    let title_ref = NodeRef::<html::Textarea>::new();
    let title_data = TextareaData {
        content: RwSignal::new(String::default()),
        textarea_ref: title_ref,
    };
    let body_ref = NodeRef::<html::Textarea>::new();
    let body_data = TextareaData {
        content: RwSignal::new(String::default()),
        textarea_ref: body_ref,
    };
    let invalid_inputs = are_satellite_inputs_invalid(title_data.content.into(), body_data.content.into());

    view! {
        <button
            class="self-end button-secondary"
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            <PlusIcon/>
        </button>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
                <div class="text-center font-bold text-2xl">"Add a satellite"</div>
                <ActionForm
                    action=sphere_state.create_satellite_action
                    on:submit=move |_| show_dialog.set(false)
                >
                    <input
                        name="sphere_name"
                        class="hidden"
                        value=sphere_state.sphere_name
                    />
                    <div class="flex flex-col gap-3 w-full">
                        <SatelliteInputs title_data body_data is_markdown_body=false is_nsfw=false is_spoiler=false/>
                        <ModalFormButtons
                            disable_publish=invalid_inputs
                            show_form=show_dialog
                        />
                    </div>
                </ActionForm>
            </div>
        </ModalDialog>
    }
}

/// Components with inputs to create or edit a satellite
#[component]
pub fn SatelliteInputs(
    title_data: TextareaData,
    body_data: TextareaData,
    is_markdown_body: bool,
    is_nsfw: bool,
    is_spoiler: bool,
) -> impl IntoView {
    view! {
        <FormTextEditor
            name="satellite_name"
            placeholder="Name"
            data=title_data
            maxlength=Some(MAX_SATELLITE_NAME_LENGTH)
        />
        <FormMarkdownEditor
            name="body"
            placeholder="Body"
            is_markdown_name="is_markdown"
            data=body_data
            is_markdown=is_markdown_body
            maxlength=Some(MAX_CONTENT_LENGTH as usize)
        />
        <LabeledFormCheckbox name="is_spoiler" label="Spoiler" value=is_spoiler/>
        <LabeledFormCheckbox name="is_nsfw" label="NSFW content" value=is_nsfw/>
    }
}

fn are_satellite_inputs_invalid(satellite_name: Signal<String>, satellite_body: Signal<String>) -> Signal<bool> {
    Signal::derive(move || {
        check_satellite_name(&*satellite_name.read()).is_err() ||
            check_string_length(
                &*satellite_body.read(),
                "Satellite body",
                MAX_CONTENT_LENGTH as usize,
                false
            ).is_err()
    })
}

#[cfg(test)]
mod test {
    use leptos::prelude::{GetUntracked, Owner, RwSignal, Set};
    use sharesphere_utils::constants::{MAX_CONTENT_LENGTH, MAX_SATELLITE_NAME_LENGTH};
    use crate::satellite::are_satellite_inputs_invalid;

    #[test]
    fn test_are_satellite_inputs_invalid() {
        let owner = Owner::new();
        owner.set();
        let satellite_name = RwSignal::new(String::from("satellite"));
        let satellite_body = RwSignal::new(String::from("body"));
        let invalid_inputs = are_satellite_inputs_invalid(satellite_name.into(), satellite_body.into());
        assert_eq!(invalid_inputs.get_untracked(), false);

        satellite_name.set(String::from("satellite name"));
        assert_eq!(invalid_inputs.get_untracked(), true);

        satellite_name.set(String::from("satellite%name"));
        assert_eq!(invalid_inputs.get_untracked(), true);

        satellite_name.set(String::from(&"a".repeat(MAX_SATELLITE_NAME_LENGTH + 1)));
        assert_eq!(invalid_inputs.get_untracked(), true);

        satellite_name.set(String::from(&"a".repeat(MAX_SATELLITE_NAME_LENGTH)));
        assert_eq!(invalid_inputs.get_untracked(), false);

        satellite_name.set(String::from("this_isA-valid_satellite"));
        assert_eq!(invalid_inputs.get_untracked(), false);

        satellite_body.set(String::from("This is a valid satellite body, it can contain special ch@racter$.\nAnd span multiple lines."));
        assert_eq!(invalid_inputs.get_untracked(), false);

        satellite_body.set(String::from(&"a".repeat(MAX_CONTENT_LENGTH as usize)));
        assert_eq!(invalid_inputs.get_untracked(), false);

        satellite_body.set(String::from(&"a".repeat(MAX_CONTENT_LENGTH  as usize + 1)));
        assert_eq!(invalid_inputs.get_untracked(), true);
    }
}