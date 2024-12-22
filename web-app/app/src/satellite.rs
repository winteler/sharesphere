use leptos::html;
use leptos::prelude::*;
use leptos_use::use_textarea_autosize;
use serde::{Deserialize, Serialize};
use server_fn::ServerFnError;

use crate::editor::{FormMarkdownEditor, FormTextEditor, TextareaData};
use crate::errors::AppError;
use crate::form::LabeledFormCheckbox;
use crate::icons::{DeleteIcon, EditIcon, NsfwIcon, PauseIcon, PlayIcon, PlusIcon, SpoilerIcon};
use crate::role::{AuthorizedShow, PermissionLevel};
use crate::sphere::SphereState;
use crate::unpack::TransitionUnpack;
use crate::widget::{ModalDialog, ModalFormButtons};

#[cfg(feature = "ssr")]
use crate::{
    app::ssr::get_db_pool,
    auth::ssr::check_user,
    editor::ssr::get_html_and_markdown_bodies,
    satellite::ssr::get_active_satellite_vec_by_sphere_name,
};

pub const SATELLITE_ROUTE_PREFIX: &str = "/satellite";
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

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::errors::AppError;
    use crate::role::PermissionLevel;
    use crate::satellite::Satellite;
    use crate::sphere::Sphere;
    use crate::user::User;
    use sqlx::PgPool;

    pub async fn get_satellite_by_id(satellite_id: i64, db_pool: &PgPool) -> Result<Satellite, AppError> {
        let satellite = sqlx::query_as!(
            Satellite,
            "SELECT * FROM satellites
            WHERE
                sphere_id = $1",
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
            WHERE
                sphere_name = $1
            ORDER BY satellite_name",
            sphere_name
        )
            .fetch_all(db_pool)
            .await?;

        Ok(satellite_vec)
    }

    pub async fn get_satellite_sphere(satellite_id: i64, db_pool: &PgPool) -> Result<Sphere, AppError> {
        let sphere = sqlx::query_as!(
            Sphere,
            "SELECT s.* FROM spheres s
            JOIN satellites sa ON sa.sphere_id = s.sphere_id
            WHERE sa.satellite_id = $1",
            satellite_id
        )
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

/// Component to display a satellite and its content
#[component]
pub fn SatelliteContent() -> impl IntoView {
    
}

/// Component to display a post inside a Satellite
#[component]
pub fn SatellitePost() -> impl IntoView {

}

/// Component to display active satellites for the current sphere
#[component]
pub fn ActiveSatelliteList() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();

    view! {
        <TransitionUnpack resource=sphere_state.satellite_resource let:satellite_vec>
        {
            match satellite_vec.is_empty() {
                true => None,
                false => {
                    let satellite_list = satellite_vec.iter().map(|satellite| {
                        let satellite_name = satellite.satellite_name.clone();
                        view! {
                            <div class="p-2 border border-1 border-base-content/20 rounded hover:bg-base-content/20 flex flex-col gap-1">
                                {satellite_name}
                                <div class="flex gap-1">
                                {
                                    match satellite.is_spoiler {
                                        true => Some(view! { <div class="h-fit w-fit px-1 py-0.5 bg-black rounded-full"><SpoilerIcon/></div> }),
                                        false => None
                                    }
                                }
                                {
                                    match satellite.is_nsfw {
                                        true => Some(view! { <NsfwIcon/>}),
                                        false => None
                                    }
                                }
                                </div>
                            </div>
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
    let satellite_resource = Resource::new(
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
        <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded">
            <div class="text-xl text-center">"Satellites"</div>
            <div class="flex flex-col gap-1">
                <div class="border-b border-base-content/20 pl-1">
                    <div class="w-5/6 flex gap-1">
                        <div class="w-3/6 py-2 font-bold">"Title"</div>
                        <div class="w-20 py-2 font-bold text-center">"Active"</div>
                        <div class="w-20 py-2 font-bold text-center">"Link"</div>
                    </div>
                </div>
                <TransitionUnpack resource=satellite_resource let:satellite_vec>
                {
                    satellite_vec.iter().map(|satellite| {
                        let show_edit_form = RwSignal::new(false);
                        let satellite_name = satellite.satellite_name.clone();
                        let satellite = satellite.clone();
                        view! {
                            <div class="flex gap-1 justify-between rounded pl-1">
                                <div class="w-5/6 flex gap-1">
                                    <div class="w-3/6 select-none">{satellite_name}</div>
                                    <div class="w-20 flex justify-center">
                                    {
                                        match satellite.disable_timestamp.is_none() {
                                            true => view! { <PlayIcon/> }.into_any(),
                                            false => view! { <PauseIcon/> }.into_any(),
                                        }
                                    }
                                    </div>
                                    <div class="w-20 flex justify-center">"Link"</div>
                                </div>
                                <div class="flex gap-1 justify-end">
                                    <button
                                        class="h-fit p-1 text-sm bg-secondary rounded-sm hover:bg-secondary/75 active:scale-90 transition duration-250"
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
                <button class="p-1 rounded-sm bg-error hover:bg-error/75 active:scale-90 transition duration-250">
                    <DeleteIcon/>
                </button>
            </ActionForm>
        </AuthorizedShow>
    }
}

/// Component to edit a sphere rule
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
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Edit a rule"</div>
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
            class="self-end p-1 bg-secondary rounded-sm hover:bg-secondary/75 active:scale-90 transition duration-250"
            on:click=move |_| show_dialog.update(|value| *value = !*value)
        >
            <PlusIcon/>
        </button>
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Add a rule"</div>
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