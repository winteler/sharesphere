use leptos::prelude::*;
use leptos_use::{use_interval};
use sharesphere_auth::auth::EndSession;
use crate::notification::{Notification, get_notifications};
use sharesphere_auth::role::{PermissionLevel, SetUserSphereRole, UserSphereRole};
use sharesphere_auth::user::{DeleteUser, SetUserSettings, User};
use sharesphere_utils::errors::AppError;
use crate::filter::SphereCategoryFilter;
use crate::moderation::ModeratePost;
use crate::post::{DeletePost, EditPost};
use crate::ranking::{CommentSortType, PostSortType, SortType};
use crate::rule::{get_rule_vec, AddRule, RemoveRule, Rule, UpdateRule};
use crate::satellite::{CreateSatellite, DisableSatellite, Satellite, UpdateSatellite};
use crate::sphere::{CreateSphere, SphereWithUserInfo, Subscribe, Unsubscribe, UpdateSphereDescription};
use crate::sphere_category::{DeleteSphereCategory, SetSphereCategory, SphereCategory};

#[derive(Copy, Clone)]
pub struct GlobalState {
    pub logout_action: ServerAction<EndSession>,
    pub delete_user_action: ServerAction<DeleteUser>,
    pub set_settings_action: ServerAction<SetUserSettings>,
    pub subscribe_action: ServerAction<Subscribe>,
    pub unsubscribe_action: ServerAction<Unsubscribe>,
    pub edit_post_action: ServerAction<EditPost>,
    pub delete_post_action: ServerAction<DeletePost>,
    pub create_sphere_action: ServerAction<CreateSphere>,
    pub sphere_reload_signal: RwSignal<usize>,
    pub post_sort_type: RwSignal<SortType>,
    pub comment_sort_type: RwSignal<SortType>,
    pub show_left_sidebar: RwSignal<bool>,
    pub show_right_sidebar: RwSignal<bool>,
    pub notif_reload_trigger: RwSignal<u64>,
    pub user: Resource<Result<Option<User>, AppError>>,
    pub notifications: Resource<Result<Vec<Notification>, AppError>>,
    pub base_rules: OnceResource<Result<Vec<Rule>, AppError>>,
}

#[derive(Copy, Clone)]
pub struct SphereState {
    pub sphere_name: Memo<String>,
    pub sphere_category_filter: RwSignal<SphereCategoryFilter>,
    pub post_refresh_count: RwSignal<usize>,
    pub permission_level: Signal<PermissionLevel>,
    pub sphere_with_user_info_resource: Resource<Result<SphereWithUserInfo, AppError>>,
    pub satellite_vec_resource: Resource<Result<Vec<Satellite>, AppError>>,
    pub sphere_categories_resource: Resource<Result<Vec<SphereCategory>, AppError>>,
    pub sphere_roles_resource: Resource<Result<Vec<UserSphereRole>, AppError>>,
    pub sphere_rules_resource: Resource<Result<Vec<Rule>, AppError>>,
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
        user: Resource<Result<Option<User>, AppError>>,
        logout_action: ServerAction<EndSession>,
        delete_user_action: ServerAction<DeleteUser>,
        create_sphere_action: ServerAction<CreateSphere>,
        set_settings_action: ServerAction<SetUserSettings>,
    ) -> Self {
        let interval_return  = use_interval(600000);
        let notif_reload_trigger = RwSignal::new(0);

        Self {
            logout_action,
            delete_user_action,
            set_settings_action,
            subscribe_action: ServerAction::<Subscribe>::new(),
            unsubscribe_action: ServerAction::<Unsubscribe>::new(),
            edit_post_action: ServerAction::<EditPost>::new(),
            delete_post_action: ServerAction::<DeletePost>::new(),
            create_sphere_action,
            sphere_reload_signal: RwSignal::new(0),
            post_sort_type: RwSignal::new(SortType::Post(PostSortType::Hot)),
            comment_sort_type: RwSignal::new(SortType::Comment(CommentSortType::Best)),
            show_left_sidebar: RwSignal::new(false),
            show_right_sidebar: RwSignal::new(false),
            notif_reload_trigger,
            user,
            notifications: Resource::new(
                move || (interval_return.counter.get(), notif_reload_trigger.get()),
                move |_| get_notifications(),
            ),
            base_rules: OnceResource::new(get_rule_vec(None))
        }
    }
}