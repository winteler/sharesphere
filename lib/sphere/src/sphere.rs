use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos_router::components::{Form, Outlet, A};
use leptos_router::hooks::{use_params_map};
use leptos_use::{signal_debounced, use_textarea_autosize};

use sharesphere_utils::editor::{FormTextEditor, TextareaData};
use sharesphere_utils::form::LabeledFormCheckbox;
use sharesphere_utils::icons::{InternalErrorIcon, LoadingIcon, MagnifierIcon, PlusIcon, SettingsIcon, SphereIcon, SubscribedIcon};
use sharesphere_utils::routes::{get_create_post_path, get_satellite_path, get_sphere_name_memo, get_sphere_path, CREATE_POST_ROUTE, CREATE_POST_SPHERE_QUERY_PARAM, CREATE_POST_SUFFIX, PUBLISH_ROUTE, SEARCH_ROUTE};
use sharesphere_utils::unpack::{handle_additional_load, handle_initial_load, ActionError, SuspenseUnpack, TransitionUnpack};

use sharesphere_auth::auth::{LoginGuardButton, LoginGuardedButton};
use sharesphere_auth::role::{get_sphere_role_vec, AuthorizedShow, PermissionLevel, SetUserSphereRole};
use sharesphere_core::filter::{PostFiltersButton, SphereCategoryFilter};
use sharesphere_core::ranking::{PostSortWidget, SortType};
use sharesphere_core::moderation::ModeratePost;
use sharesphere_core::post::{add_sphere_info_to_post_vec, get_post_vec_by_sphere_name, PostMiniatureList, PostWithSphereInfo};
use sharesphere_core::rule::{get_sphere_rule_vec, AddRule, RemoveRule, UpdateRule};
use sharesphere_core::satellite::{CreateSatellite, DisableSatellite, UpdateSatellite};
use sharesphere_core::sidebar::SphereSidebar;
use sharesphere_core::satellite::get_satellite_vec_by_sphere_name;
use sharesphere_core::sphere::{get_sphere_by_name, get_sphere_with_user_info, is_sphere_available, is_valid_sphere_name, SphereWithUserInfo, Subscribe, Unsubscribe, UpdateSphereDescription};
use sharesphere_core::sphere_category::{get_sphere_category_vec, DeleteSphereCategory, SetSphereCategory};
use sharesphere_core::state::{GlobalState, SphereState};

use crate::satellite::{ActiveSatelliteList, SatelliteState};
use crate::sphere_category::{get_sphere_category_header_map};
use crate::sphere_management::MANAGE_SPHERE_ROUTE;

/// Component to display a sphere's banner
#[component]
pub fn SphereBanner() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_name = get_sphere_name_memo(use_params_map());
    let create_satellite_action = ServerAction::<CreateSatellite>::new();
    let update_satellite_action = ServerAction::<UpdateSatellite>::new();
    let disable_satellite_action = ServerAction::<DisableSatellite>::new();
    let update_sphere_desc_action = ServerAction::<UpdateSphereDescription>::new();
    let set_sphere_category_action = ServerAction::<SetSphereCategory>::new();
    let delete_sphere_category_action = ServerAction::<DeleteSphereCategory>::new();
    let set_sphere_role_action = ServerAction::<SetUserSphereRole>::new();
    let add_rule_action = ServerAction::<AddRule>::new();
    let update_rule_action = ServerAction::<UpdateRule>::new();
    let remove_rule_action = ServerAction::<RemoveRule>::new();
    let sphere_state = SphereState {
        sphere_name,
        sphere_category_filter: RwSignal::new(SphereCategoryFilter::All),
        permission_level: Signal::derive(
            move || match &(*state.user.read()) {
                Some(Ok(Some(user))) => user.get_sphere_permission_level(&*sphere_name.read()),
                _ => PermissionLevel::None,
            }
        ),
        sphere_resource: Resource::new(
            move || (
                sphere_name.get(),
                update_sphere_desc_action.version().get(),
                state.sphere_reload_signal.get(),
            ),
            move |(sphere_name, _, _)| get_sphere_by_name(sphere_name)
        ),
        satellite_vec_resource: Resource::new(
            move || (
                sphere_name.get(),
                create_satellite_action.version().get(),
                update_satellite_action.version().get(),
                disable_satellite_action.version().get(),
            ),
            move |(sphere_name, _, _, _)| get_satellite_vec_by_sphere_name(sphere_name, true)
        ),
        sphere_categories_resource: Resource::new(
            move || (
                sphere_name.get(),
                set_sphere_category_action.version().get(),
                delete_sphere_category_action.version().get()
            ),
            move |(sphere_name, _, _)| get_sphere_category_vec(sphere_name)
        ),
        sphere_roles_resource: Resource::new(
            move || (sphere_name.get(), set_sphere_role_action.version().get()),
            move |(sphere_name, _)| get_sphere_role_vec(sphere_name),
        ),
        sphere_rules_resource: Resource::new(
            move || (
                sphere_name.get(),
                add_rule_action.version().get(),
                update_rule_action.version().get(),
                remove_rule_action.version().get()
            ),
            move |(sphere_name, _, _, _)| get_sphere_rule_vec(sphere_name),
        ),
        create_satellite_action,
        update_satellite_action,
        disable_satellite_action,
        moderate_post_action: ServerAction::<ModeratePost>::new(),
        update_sphere_desc_action,
        set_sphere_category_action,
        delete_sphere_category_action,
        set_sphere_role_action,
        add_rule_action,
        update_rule_action,
        remove_rule_action,
    };
    provide_context(sphere_state);

    let sphere_path = move || get_sphere_path(&sphere_name.get());

    view! {
        <div class="flex flex-col gap-2 pt-2 px-2 w-full">
            <TransitionUnpack resource=sphere_state.sphere_resource let:sphere>
            {
                let sphere_banner_class = format!(
                    "flex-none bg-cover bg-left bg-no-repeat bg-[url('{}')] rounded-sm w-full h-16 2xl:h-40 flex items-center justify-center",
                    sphere.banner_url.clone().unwrap_or(String::from("/banner.jpg"))
                );
                view! {
                    <a
                        href=sphere_path()
                        class=sphere_banner_class
                    >
                        <div class="p-3 backdrop-blur-sm bg-black/50 rounded-xs flex justify-center gap-3">
                            <SphereIcon icon_url=sphere.icon_url.clone() class="h-8 w-8 2xl:h-12 2xl:w-12"/>
                            <span class="text-2xl 2xl:text-4xl">{sphere_state.sphere_name.get()}</span>
                        </div>
                    </a>
                }.into_any()
            }
            </TransitionUnpack>
            <Outlet/>
        </div>
        <div class="max-2xl:hidden">
            <SphereSidebar/>
        </div>
    }.into_any()
}

/// Component to display a sphere's contents
#[component]
pub fn SphereContents() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = expect_context::<SphereState>().sphere_name;
    let additional_load_count = RwSignal::new(0);
    let post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();
    let sphere_with_sub_resource = Resource::new(
        move || (sphere_name(),),
        move |(sphere_name,)| get_sphere_with_user_info(sphere_name),
    );

    let _initial_post_resource = LocalResource::new(
        move || async move {
            post_vec.write().clear();
            is_loading.set(true);
            // TODO return map in resource directly?
            let sphere_category_map = get_sphere_category_header_map(sphere_state.sphere_categories_resource.await);
            // TODO check no unnecessary loads
            let initial_load = get_post_vec_by_sphere_name(
                sphere_name.get(),
                sphere_state.sphere_category_filter.get(),
                state.post_sort_type.get(),
                0,
            ).await.map(|post_vec| add_sphere_info_to_post_vec(post_vec, sphere_category_map, None));
            handle_initial_load(initial_load, post_vec, load_error, Some(list_ref));
            is_loading.set(false);
        }
    );

    let _additional_post_resource = LocalResource::new(
        move || async move {
            if additional_load_count.get() > 0 {
                is_loading.set(true);
                let sphere_category_map = get_sphere_category_header_map(sphere_state.sphere_categories_resource.await);
                let num_post = post_vec.read_untracked().len();
                let additional_load = get_post_vec_by_sphere_name(
                    sphere_name.get_untracked(),
                    sphere_state.sphere_category_filter.get_untracked(),
                    state.post_sort_type.get_untracked(),
                    num_post
                ).await.map(|post_vec| add_sphere_info_to_post_vec(post_vec, sphere_category_map, None));
                handle_additional_load(additional_load, post_vec, load_error);
                is_loading.set(false);
            }
        }
    );

    view! {
        <ActiveSatelliteList/>
        <SuspenseUnpack resource=sphere_with_sub_resource let:sphere>
            <SphereToolbar
                sphere
                sort_signal=state.post_sort_type
            />
        </SuspenseUnpack>
        <PostMiniatureList
            post_vec
            is_loading
            load_error
            additional_load_count
            list_ref
            show_sphere_header=false
        />
    }.into_any()
}

/// Component to display the sphere toolbar
#[component]
pub fn SphereToolbar<'a>(
    sphere: &'a SphereWithUserInfo,
    sort_signal: RwSignal<SortType>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();
    let satellite_state = use_context::<SatelliteState>();
    let sphere_id = sphere.sphere.sphere_id;
    let sphere_name = RwSignal::new(sphere.sphere.sphere_name.clone());
    let is_subscribed = RwSignal::new(sphere.subscription_id.is_some());
    let manage_path = move || get_sphere_path(&sphere_name.get()) + MANAGE_SPHERE_ROUTE;

    view! {
        <div class="flex w-full justify-between items-center">
            <div class="flex items-center w-full 2xl:gap-2">
                <PostSortWidget sort_signal/>
                <PostFiltersButton/>
            </div>
            <div class="flex items-center 2xl:gap-1">
                <AuthorizedShow sphere_name permission_level=PermissionLevel::Moderate>
                    <A href=manage_path attr:class="btn btn-circle btn-ghost max-2xl:btn-sm">
                        <SettingsIcon class="h-4 w-4 2xl:h-5 2xl:w-5"/>
                    </A>
                </AuthorizedShow>
                <SphereSearchButton/>
                <div class="tooltip" data-tip="New">
                    <LoginGuardButton
                        login_button_class="btn btn-circle btn-ghost max-2xl:btn-sm"
                        login_button_content=move || view! { <PlusIcon class="h-4 w-4 2xl:h-6 2xl:w-6"/> }.into_any()
                        redirect_path_fn=&get_create_post_path
                        let:_user
                    >
                    { move || match satellite_state {
                        Some(satellite_state) => {
                            let create_post_link = get_satellite_path(
                                &*sphere_state.sphere_name.read(),
                                satellite_state.satellite_id.get()
                            ) + PUBLISH_ROUTE + CREATE_POST_SUFFIX;
                            Either::Left(view! {
                                <a href=create_post_link class="btn btn-circle btn-ghost max-2xl:btn-sm">
                                    <PlusIcon class="h-4 w-4 2xl:h-6 2xl:w-6"/>
                                </a>
                            })
                        }
                        None => Either::Right(view! {
                            <Form method="GET" action=CREATE_POST_ROUTE attr:class="flex">
                                <input type="text" name=CREATE_POST_SPHERE_QUERY_PARAM class="hidden" value=sphere_name/>
                                <button type="submit" class="btn btn-circle btn-ghost max-2xl:btn-sm">
                                    <PlusIcon class="h-4 w-4 2xl:h-6 2xl:w-6"/>
                                </button>
                            </Form>
                        }),
                    }}
                    </LoginGuardButton>
                </div>
                <div class="tooltip" data-tip="Join">
                    <LoginGuardedButton
                        button_class="btn btn-circle btn-ghost max-2xl:btn-sm"
                        button_action=move |_| {
                            is_subscribed.update(|value| {
                                *value = !*value;
                                if *value {
                                    state.subscribe_action.dispatch(Subscribe { sphere_id });
                                } else {
                                    state.unsubscribe_action.dispatch(Unsubscribe { sphere_id });
                                }
                            })
                        }
                    >
                        <SubscribedIcon class="h-4 w-4 2xl:h-6 2xl:w-6" show_color=is_subscribed/>
                    </LoginGuardedButton>
                </div>
            </div>
        </div>
    }.into_any()
}

/// Component to create new spheres
#[component]
pub fn CreateSphere() -> impl IntoView {
    let state = expect_context::<GlobalState>();

    let sphere_name = RwSignal::new(String::new());
    let sphere_name_debounced: Signal<String> = signal_debounced(sphere_name, 250.0);
    let is_sphere_available = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_name| async {
            if sphere_name.is_empty() {
                None
            } else {
                Some(is_sphere_available(sphere_name).await)
            }
        },
    );

    let is_name_taken = RwSignal::new(false);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let textarea_autosize = use_textarea_autosize(textarea_ref);
    let description_data = TextareaData {
        content: textarea_autosize.content,
        set_content: textarea_autosize.set_content,
        textarea_ref,
    };
    let is_name_empty = move || sphere_name.read().is_empty();
    let is_name_alphanumeric =
        move || is_valid_sphere_name(&sphere_name.read());
    let are_inputs_invalid = Memo::new(move |_| {
        is_name_empty()
            || is_name_taken.get()
            || !is_name_alphanumeric()
            || description_data.content.read().is_empty()
    });

    view! {
        <div class="w-4/5 2xl:w-2/5 p-2 mx-auto flex flex-col gap-2 overflow-auto">
            <ActionForm action=state.create_sphere_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Settle a Sphere!"</h2>
                    <div class="h-full flex gap-2">
                        <input
                            type="text"
                            name="sphere_name"
                            placeholder="Name"
                            autocomplete="off"
                            class="input input-primary flex-none w-3/5"
                            autofocus
                            on:input=move |ev| {
                                sphere_name.set(event_target_value(&ev));
                            }
                            prop:value=sphere_name
                        />
                        <Suspense fallback=move || view! { <LoadingIcon class="h-7 w-7"/> }>
                        {
                            move || is_sphere_available.map(|result| match result {
                                None | Some(Ok(true)) => {
                                    is_name_taken.set(false);
                                    view! {}.into_any()
                                },
                                Some(Ok(false)) => {
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error flex items-center justify-center">
                                            <span class="font-semibold">"Unavailable"</span>
                                        </div>
                                    }.into_any()
                                },
                                Some(Err(e)) => {
                                    log::error!("Error while checking sphere existence: {e}");
                                    is_name_taken.set(true);
                                    view! {
                                        <div class="alert alert-error h-fit py-2 flex items-center justify-center">
                                            <InternalErrorIcon class="h-16 w-16"/>
                                            <span class="font-semibold">"Server error"</span>
                                        </div>
                                    }.into_any()
                                },
                            })

                        }
                        </Suspense>
                        <div class="alert alert-error flex items-center" class:hidden=move || is_name_empty() || is_name_alphanumeric()>
                            <InternalErrorIcon class="h-16 w-16"/>
                            <span>"Only alphanumeric characters."</span>
                        </div>
                    </div>
                    <FormTextEditor
                        name="description"
                        placeholder="Description"
                        data=description_data
                    />
                    <LabeledFormCheckbox name="is_nsfw" label="NSFW content"/>
                    <Suspense fallback=move || view! { <LoadingIcon/> }>
                        <button type="submit" class="btn btn-secondary" disabled=are_inputs_invalid>"Create"</button>
                    </Suspense>
                </div>
            </ActionForm>
            <ActionError action=state.create_sphere_action.into()/>
        </div>
    }
}

/// Button to navigate to the search page of a sphere
#[component]
pub fn SphereSearchButton() -> impl IntoView
{
    let sphere_state = expect_context::<SphereState>();
    let route = move || format!("{}{}", get_sphere_path(sphere_state.sphere_name.read_untracked().as_str()), SEARCH_ROUTE);
    view! {
        <a href=route class="btn btn-ghost btn-circle max-2xl:btn-sm">
            <MagnifierIcon class="h-4 w-4 2xl:h-6 2xl:w-6"/>
        </a>
    }
}