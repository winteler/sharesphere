use leptos::prelude::{Memo, Resource, RwSignal, ServerAction, Signal};
use server_fn::ServerFnError;
use sharesphere_auth::auth::EndSession;
use sharesphere_auth::role::{PermissionLevel, SetUserSphereRole, UserSphereRole};
use sharesphere_auth::user::{SetUserSettings, User};
use sharesphere_utils::errors::AppError;
use crate::moderation::ModeratePost;
use crate::post::{DeletePost, EditPost};
use crate::ranking::{CommentSortType, PostSortType, SortType};
use crate::rule::{AddRule, RemoveRule, Rule, UpdateRule};
use crate::satellite::{CreateSatellite, DisableSatellite, Satellite, UpdateSatellite};
use crate::sphere::{CreateSphere, Sphere, Subscribe, Unsubscribe, UpdateSphereDescription};
use crate::sphere_category::{DeleteSphereCategory, SetSphereCategory, SphereCategory};

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub logout_action: ServerAction<EndSession>,
    pub set_settings_action: ServerAction<SetUserSettings>,
    pub subscribe_action: ServerAction<Subscribe>,
    pub unsubscribe_action: ServerAction<Unsubscribe>,
    pub edit_post_action: ServerAction<EditPost>,
    pub delete_post_action: ServerAction<DeletePost>,
    pub create_sphere_action: ServerAction<CreateSphere>,
    pub sphere_reload_signal: RwSignal<usize>,
    pub post_sort_type: RwSignal<SortType>,
    pub comment_sort_type: RwSignal<SortType>,
    pub user: Resource<leptos::error::Result<Option<User>, ServerFnError<AppError>>>,
}

#[derive(Copy, Clone)]
pub struct SphereState {
    pub sphere_name: Memo<String>,
    pub category_id_filter: RwSignal<Option<i64>>,
    pub permission_level: Signal<PermissionLevel>,
    pub sphere_resource: Resource<leptos::error::Result<Sphere, ServerFnError<AppError>>>,
    pub satellite_vec_resource: Resource<leptos::error::Result<Vec<Satellite>, ServerFnError<AppError>>>,
    pub sphere_categories_resource: Resource<leptos::error::Result<Vec<SphereCategory>, ServerFnError<AppError>>>,
    pub sphere_roles_resource: Resource<leptos::error::Result<Vec<UserSphereRole>, ServerFnError<AppError>>>,
    pub sphere_rules_resource: Resource<leptos::error::Result<Vec<Rule>, ServerFnError<AppError>>>,
    pub create_satellite_action: ServerAction<CreateSatellite>,
    pub update_satellite_action: ServerAction<UpdateSatellite>,
    pub disable_satellite_action: ServerAction<DisableSatellite>,
    pub moderate_post_action: ServerAction<ModeratePost>,
    pub update_sphere_desc_action: ServerAction<UpdateSphereDescription>,
    pub set_sphere_category_action: ServerAction<SetSphereCategory>,
    pub delete_sphere_category_action: ServerAction<DeleteSphereCategory>,
    pub set_sphere_role_action: ServerAction<SetUserSphereRole>,
    pub add_rule_action: ServerAction<AddRule>,
    pub update_rule_action: ServerAction<UpdateRule>,
    pub remove_rule_action: ServerAction<RemoveRule>,
}

impl GlobalState {
    pub fn new(
        user: Resource<leptos::error::Result<Option<User>, ServerFnError<AppError>>>,
        logout_action: ServerAction<EndSession>,
        create_sphere_action: ServerAction<CreateSphere>,
        set_settings_action: ServerAction<SetUserSettings>,
    ) -> Self {
        Self {
            logout_action,
            set_settings_action,
            subscribe_action: ServerAction::<Subscribe>::new(),
            unsubscribe_action: ServerAction::<Unsubscribe>::new(),
            edit_post_action: ServerAction::<EditPost>::new(),
            delete_post_action: ServerAction::<DeletePost>::new(),
            create_sphere_action,
            sphere_reload_signal: RwSignal::new(0),
            post_sort_type: RwSignal::new(SortType::Post(PostSortType::Hot)),
            comment_sort_type: RwSignal::new(SortType::Comment(CommentSortType::Best)),
            user,
        }
    }
}