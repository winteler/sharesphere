use leptos::html;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_params_map;
use leptos_router::params::ParamsMap;
use leptos_use::use_textarea_autosize;
use serde::{Deserialize, Serialize};
use server_fn::ServerFnError;

use utils::editor::{FormMarkdownEditor, FormTextEditor, TextareaData};
use utils::embed::EmbedType;
use utils::errors::AppError;
use utils::form::LabeledFormCheckbox;
use utils::icons::{CrossIcon, EditIcon, LinkIcon, PauseIcon, PlayIcon, PlusIcon};
use utils::unpack::{handle_additional_load, handle_initial_load, ActionError, SuspenseUnpack, TransitionUnpack};
use utils::role::{AuthorizedShow, PermissionLevel};
use utils::widget::{ModalDialog, ModalFormButtons, TagsWidget};

use crate::content::ContentBody;
use crate::post::{add_sphere_info_to_post_vec, get_post_vec_by_satellite_id, CreatePost, PostForm, PostMiniatureList, PostSortType, PostWithSphereInfo};
use crate::ranking::SortType;
use crate::sphere::{get_sphere_with_user_info, SphereState, SphereToolbar, SPHERE_ROUTE_PREFIX};
use crate::sphere_category::{get_sphere_category_header_map, get_sphere_category_vec};

#[cfg(feature = "ssr")]
use {
    utils::{
        auth::ssr::check_user,
        editor::ssr::get_html_and_markdown_bodies,
        utils::ssr::get_db_pool,
    },
    crate::{
        satellite::ssr::get_active_satellite_vec_by_sphere_name
    },
};


pub const SATELLITE_ROUTE_PREFIX: &str = "/satellites";
pub const SATELLITE_ROUTE_PARAM_NAME: &str = "satellite_id";

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Satellite {
    pub satellite_id: i64,
    pub satellite_name: String,
    pub sphere_id: i64,
    pub sphere_name: String,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_nsfw: bool,
    pub is_spoiler: bool,
    pub num_posts: i32,
    pub creator_id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub disable_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone)]
pub struct SatelliteState {
    pub satellite_id: Memo<i64>,
    pub sort_type: RwSignal<SortType>,
    pub category_id_filter: RwSignal<Option<i64>>,
    pub satellite_resource: Resource<Result<Satellite, ServerFnError<AppError>>>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use utils::errors::AppError;
    use utils::role::PermissionLevel;
    use utils::user::User;

    use crate::satellite::Satellite;
    use crate::sphere::Sphere;


    pub async fn get_satellite_by_id(satellite_id: i64, db_pool: &PgPool) -> Result<Satellite, AppError> {
        let satellite = sqlx::query_as!(
            Satellite,
            "SELECT * FROM satellites
            WHERE satellite_id = $1",
            satellite_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }
    
    pub async fn get_active_satellite_vec_by_sphere_name(sphere_name: &str, db_pool: &PgPool) -> Result<Vec<Satellite>, AppError> {
        let satellite_vec = sqlx::query_as!(
            Satellite,
            "SELECT * FROM satellites
            WHERE
                sphere_name = $1 AND
                disable_timestamp IS NULL
            ORDER BY satellite_name",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(satellite_vec)
    }

    pub async fn get_satellite_vec_by_sphere_name(sphere_name: &str, db_pool: &PgPool) -> Result<Vec<Satellite>, AppError> {
        let satellite_vec = sqlx::query_as!(
            Satellite,
            "SELECT * FROM satellites
            WHERE sphere_name = $1
            ORDER BY satellite_name",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(satellite_vec)
    }

    pub async fn get_satellite_sphere(satellite_id: i64, db_pool: &PgPool) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as::<_, Sphere>(
            "SELECT s.* FROM spheres s
            JOIN satellites sa ON sa.sphere_id = s.sphere_id
            WHERE sa.satellite_id = $1"
        )
            .bind(satellite_id)
            .fetch_one(db_pool)
            .await?;

        Ok(sphere)
    }

    pub async fn create_satellite(
        sphere_name: &str,
        satellite_name: &str,
        body: &str,
        markdown_body: Option<&str>,
        is_nsfw: bool,
        is_spoiler: bool,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        user.check_permissions(sphere_name, PermissionLevel::Manage)?;
        
        let satellite = sqlx::query_as!(
            Satellite,
            "INSERT INTO satellites
            (satellite_name, sphere_id, sphere_name, body, markdown_body, is_nsfw, is_spoiler, creator_id)
            VALUES (
                $1,
                (SELECT sphere_id FROM spheres WHERE sphere_name = $2),
                $2, $3, $4,
                (
                    CASE 
                        WHEN $5 THEN TRUE
                        ELSE (SELECT is_nsfw FROM spheres WHERE sphere_name = $2)
                    END
                ),
                $6, $7
            ) 
            RETURNING *",
            satellite_name,
            sphere_name,
            body,
            markdown_body,
            is_nsfw,
            is_spoiler,
            user.user_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }

    pub async fn update_satellite(
        satellite_id: i64,
        satellite_name: &str,
        body: &str,
        markdown_body: Option<&str>,
        is_nsfw: bool,
        is_spoiler: bool,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        let sphere = get_satellite_sphere(satellite_id, db_pool).await?;
        user.check_permissions(&sphere.sphere_name, PermissionLevel::Manage)?;

        let satellite = sqlx::query_as!(
            Satellite,
            "UPDATE satellites
            SET
                satellite_name = $1,
                body = $2,
                markdown_body = $3,
                is_nsfw = $4,
                is_spoiler = $5
            WHERE satellite_id = $6
            RETURNING *",
            satellite_name,
            body,
            markdown_body,
            is_nsfw || sphere.is_nsfw,
            is_spoiler,
            satellite_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }

    pub async fn disable_satellite(
        satellite_id: i64,
        user: &User,
        db_pool: &PgPool
    ) -> Result<Satellite, AppError> {
        let sphere = get_satellite_sphere(satellite_id, db_pool).await?;
        user.check_permissions(&sphere.sphere_name, PermissionLevel::Manage)?;

        let satellite = sqlx::query_as!(
            Satellite,
            "UPDATE satellites
            SET disable_timestamp = CURRENT_TIMESTAMP
            WHERE satellite_id = $1
            RETURNING *",
            satellite_id,
        )
            .fetch_one(db_pool)
            .await?;

        Ok(satellite)
    }
}

#[server]
pub async fn get_satellite_by_id(
    satellite_id: i64,
) -> Result<Satellite, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let satellite = ssr::get_satellite_by_id(satellite_id, &db_pool).await?;
    Ok(satellite)
}

#[server]
pub async fn get_satellite_vec_by_sphere_name(
    sphere_name: String,
    only_active: bool,
) -> Result<Vec<Satellite>, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let satellite_vec = match only_active {
        true => get_active_satellite_vec_by_sphere_name(&sphere_name, &db_pool).await?,
        false => ssr::get_satellite_vec_by_sphere_name(&sphere_name, &db_pool).await?,
    };
    Ok(satellite_vec)
}

#[server]
pub async fn create_satellite(
    sphere_name: String,
    satellite_name: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    is_spoiler: bool,
) -> Result<Satellite, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    let (body, markdown_body) = get_html_and_markdown_bodies(body, is_markdown).await?;

    let satellite = ssr::create_satellite(
        &sphere_name,
        &satellite_name,
        &body,
        markdown_body.as_deref(),
        is_nsfw,
        is_spoiler,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}

#[server]
pub async fn update_satellite(
    satellite_id: i64,
    satellite_name: String,
    body: String,
    is_markdown: bool,
    is_nsfw: bool,
    is_spoiler: bool,
) -> Result<Satellite, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;

    let (body, markdown_body) = get_html_and_markdown_bodies(body, is_markdown).await?;

    let satellite = ssr::update_satellite(
        satellite_id,
        &satellite_name,
        &body,
        markdown_body.as_deref(),
        is_nsfw,
        is_spoiler,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}

#[server]
pub async fn disable_satellite(
    satellite_id: i64,
) -> Result<Satellite, ServerFnError<AppError>> {
    let db_pool = get_db_pool()?;
    let user = check_user().await?;
    let satellite = ssr::disable_satellite(
        satellite_id,
        &user,
        &db_pool
    ).await?;
    Ok(satellite)
}

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
            <div class="w-1/2 2xl:w-1/4">
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
    let post_vec = RwSignal::new(Vec::<PostWithSphereInfo>::new());
    let is_loading = RwSignal::new(false);
    let load_error = RwSignal::new(None);
    let list_ref = NodeRef::<html::Ul>::new();

    let _initial_post_resource = LocalResource::new(
        move || async move {
            post_vec.write().clear();
            is_loading.set(true);
            // TODO return map in resource directly?
            let sphere_category_map = get_sphere_category_header_map(sphere_state.sphere_categories_resource.await);

            let initial_load = get_post_vec_by_satellite_id(
                satellite_state.satellite_id.get(),
                category_id_signal.get(),
                sort_signal.get(),
                0
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
                let additional_load = get_post_vec_by_satellite_id(
                    satellite_state.satellite_id.get_untracked(),
                    category_id_signal.get_untracked(),
                    sort_signal.get_untracked(),
                    num_post
                ).await.map(|post_vec| add_sphere_info_to_post_vec(post_vec, sphere_category_map, None));
                handle_additional_load(additional_load, post_vec, load_error);
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
                category_id_signal
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
    let body_autosize = use_textarea_autosize(textarea_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref,
    };
    let link_input = RwSignal::new(String::default());
    let embed_type_input = RwSignal::new(EmbedType::None);

    let category_vec_resource = Resource::new(
        move || sphere_state.sphere_name.get(),
        move |sphere_name| get_sphere_category_vec(sphere_name)
    );

    view! {
        <div class="w-4/5 2xl:w-2/5 p-2 mx-auto flex flex-col gap-2 overflow-auto">
            <ActionForm action=create_post_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Share a post!"</h2>
                    <input
                        type="text"
                        name="sphere"
                        class="hidden"
                        value=sphere_state.sphere_name
                    />
                    <input
                        type="text"
                        name="satellite_id"
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
                    <button type="submit" class="btn btn-secondary" disabled=move || {
                        title_input.read().is_empty() ||
                        (
                            body_data.content.read().is_empty() &&
                            *embed_type_input.read() == EmbedType::None
                        ) || (
                            *embed_type_input.read() != EmbedType::None &&
                            link_input.read().is_empty() // TODO check valid url?
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
                        <div class="grid grid-cols-2 2xl:grid-cols-4 gap-2">
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
        <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded-sm">
            <div class="text-xl text-center">"Satellites"</div>
            <div class="flex flex-col gap-1">
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
                                        <a href=satellite_link class="p-2 rounded-full hover:bg-base-200">
                                            <LinkIcon/>
                                        </a>
                                    </div>
                                </div>
                                <div class="flex gap-1 justify-end">
                                    <button
                                        class="h-fit p-1 text-sm bg-secondary rounded-xs hover:bg-secondary/75 active:scale-90 transition duration-250"
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
                <button class="p-1 rounded-xs bg-error hover:bg-error/75 active:scale-90 transition duration-250">
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
    let title_autosize = use_textarea_autosize(title_ref);
    let title_data = TextareaData {
        content: title_autosize.content,
        set_content: title_autosize.set_content,
        textarea_ref: title_ref,
    };
    title_data.set_content.set(satellite.satellite_name);
    let body_ref = NodeRef::<html::Textarea>::new();
    let body_autosize = use_textarea_autosize(body_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref: body_ref,
    };
    let (body, is_markdown_body) = match satellite.markdown_body {
        Some(markdown_body) => (markdown_body, true),
        None => (satellite.body, false),
    };
    body_data.set_content.set(body);
    let invalid_inputs = Signal::derive(move || {
        title_autosize.content.read().is_empty() || body_data.content.read().is_empty()
    });

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
    let title_autosize = use_textarea_autosize(title_ref);
    let title_data = TextareaData {
        content: title_autosize.content,
        set_content: title_autosize.set_content,
        textarea_ref: title_ref,
    };
    let body_ref = NodeRef::<html::Textarea>::new();
    let body_autosize = use_textarea_autosize(body_ref);
    let body_data = TextareaData {
        content: body_autosize.content,
        set_content: body_autosize.set_content,
        textarea_ref: body_ref,
    };
    let invalid_inputs = Signal::derive(move || {
        title_autosize.content.read().is_empty() || body_data.content.read().is_empty()
    });

    view! {
        <button
            class="self-end p-1 bg-secondary rounded-xs hover:bg-secondary/75 active:scale-90 transition duration-250"
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
        <div class="flex flex-col gap-1 content-center">
            <FormTextEditor
                name="satellite_name"
                placeholder="Name"
                data=title_data
            />
            <FormMarkdownEditor
                name="body"
                placeholder="Body"
                is_markdown_name="is_markdown"
                data=body_data
                is_markdown=is_markdown_body
            />
            <LabeledFormCheckbox name="is_spoiler" label="Spoiler" value=is_spoiler/>
            <LabeledFormCheckbox name="is_nsfw" label="NSFW content" value=is_nsfw/>
        </div>
    }
}

/// # Returns the path to a satellite given its id and sphere name
///
/// ```
/// use app::satellite::get_satellite_path;
///
/// assert_eq!(get_satellite_path("test", 1), "/spheres/test/satellites/1");
/// ```
pub fn get_satellite_path(
    sphere_name: &str,
    satellite_id: i64
) -> String {
    format!("{SPHERE_ROUTE_PREFIX}/{}{SATELLITE_ROUTE_PREFIX}/{}", sphere_name, satellite_id)
}

/// Get a memo returning the last valid satellite_id from the url. Used to avoid triggering resources when leaving pages
pub fn get_satellite_id_memo(params: Memo<ParamsMap>) -> Memo<i64> {
    Memo::new(move |current_satellite_id: Option<&i64>| {
        if let Some(new_satellite_id_str) = params.read().get_str(SATELLITE_ROUTE_PARAM_NAME) {
            if let Ok(new_satellite_id) = new_satellite_id_str.parse::<i64>() {
                log::trace!("Current satellite id: {current_satellite_id:?}, new satellite id: {new_satellite_id}");
                new_satellite_id
            } else {
                log::trace!("Could not parse new satellite id: {new_satellite_id_str}, reuse current satellite id: {current_satellite_id:?}");
                current_satellite_id.cloned().unwrap_or_default()
            }
        } else {
            log::trace!("Could not find new satellite id, reuse current satellite id: {current_satellite_id:?}");
            current_satellite_id.cloned().unwrap_or_default()
        }
    })
}