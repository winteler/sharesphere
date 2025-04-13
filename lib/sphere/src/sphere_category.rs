use std::collections::HashMap;
use leptos::html;
use leptos::prelude::*;

use sharesphere_utils::colors::{Color, ColorIndicator, ColorSelect};
use sharesphere_utils::editor::{FormTextEditor, TextareaData};
use sharesphere_utils::errors::AppError;
use sharesphere_utils::form::FormCheckbox;
use sharesphere_utils::icons::{CrossIcon, PauseIcon, PlayIcon, SaveIcon};
use sharesphere_utils::unpack::TransitionUnpack;

use sharesphere_auth::role::{AuthorizedShow, PermissionLevel};

use sharesphere_core::sphere_category::{SphereCategory, SphereCategoryHeader};
use sharesphere_core::state::SphereState;

/// Component to manage sphere categories
#[component]
pub fn SphereCategoriesDialog() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;

    let category_input = RwSignal::new(String::new());
    let color_input = RwSignal::new(Color::None);
    let activated_input = RwSignal::new(true);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let description_data = TextareaData {
        content: RwSignal::new(String::new()),
        textarea_ref
    };
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            // TODO add overflow-y-auto max-h-full?
            <div class="shrink-0 flex flex-col gap-1 items-center w-full h-fit bg-base-200 p-2 rounded-sm">
                <div class="text-xl text-center">"Sphere categories"</div>
                <div class="flex flex-col">
                    <div class="border-b border-base-content/20 pl-2">
                        <div class="w-5/6 flex gap-1">
                            <div class="w-3/12 py-2 font-bold">"Category"</div>
                            <div class="w-1/12 py-2 font-bold">"Color"</div>
                            <div class="w-3/6 py-2 font-bold">"Description"</div>
                            <div class="w-20 py-2 font-bold text-center">"Active"</div>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1 pl-2 py-1">
                        <TransitionUnpack resource=sphere_state.sphere_categories_resource let:sphere_category_vec>
                        {
                            sphere_category_vec.iter().map(|sphere_category| {
                                let category_name = sphere_category.category_name.clone();
                                let color = sphere_category.category_color;
                                let description = sphere_category.description.clone();
                                let is_active = sphere_category.is_active;
                                view! {
                                    <div
                                        class="flex justify-between items-center"
                                    >
                                        <div
                                            class="w-5/6 flex items-center gap-1 p-1 rounded-sm hover:bg-base-200 active:scale-95 transition duration-250"
                                            on:click=move |_| {
                                                category_input.set(category_name.clone());
                                                color_input.set(color);
                                                description_data.content.set(description.clone());
                                                if let Some(textarea_ref) = textarea_ref.get() {
                                                    textarea_ref.set_value(&description);
                                                }
                                                activated_input.set(is_active);
                                            }
                                        >
                                            <div class="w-3/12 select-none">{category_name.clone()}</div>
                                            <div class="w-1/12 h-fit"><ColorIndicator color/></div>
                                            <div class="w-3/6 select-none whitespace-pre-wrap">{description.clone()}</div>
                                            <div class="w-20 flex justify-center">
                                            {
                                                match is_active {
                                                    true => view! { <PlayIcon/> }.into_any(),
                                                    false => view! { <PauseIcon/> }.into_any(),
                                                }
                                            }
                                            </div>
                                        </div>
                                        <DeleteCategoryButton category_name=sphere_category.category_name.clone()/>
                                    </div>
                                }
                            }).collect_view()
                        }
                        </TransitionUnpack>
                    </div>
                    <SetCategoryForm category_input color_input activated_input description_data/>
                </div>
            </div>
        </AuthorizedShow>
    }
}

/// Component to set permission levels for a sphere
#[component]
pub fn SetCategoryForm(
    category_input: RwSignal<String>,
    color_input: RwSignal<Color>,
    activated_input: RwSignal<bool>,
    description_data: TextareaData,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    let disable_submit = move || category_input.read().is_empty() && description_data.content.read().is_empty();

    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            <ActionForm action=sphere_state.set_sphere_category_action>
                <input
                    name="sphere_name"
                    class="hidden"
                    value=sphere_name
                />
                <div class="w-full flex gap-1 justify-between items-stretch pl-2">
                    <div class="flex gap-1 items-center w-5/6 p-1">
                        <input
                            tabindex="0"
                            type="text"
                            name="category_name"
                            placeholder="Category"
                            autocomplete="off"
                            class="input input-primary w-3/12"
                            on:input=move |ev| {
                                category_input.set(event_target_value(&ev));
                            }
                            prop:value=category_input
                        />
                        <ColorSelect name="category_color" color_input class="w-1/12"/>
                        <FormTextEditor
                            name="description"
                            placeholder="Description"
                            data=description_data
                            class="w-3/6"
                        />
                        <FormCheckbox name="is_active" is_checked=activated_input class="w-20 self-center flex justify-center"/>
                    </div>
                    <button
                        type="submit"
                        disabled=disable_submit
                        class="button-secondary self-center"
                    >
                        <SaveIcon/>
                    </button>
                </div>
            </ActionForm>
        </AuthorizedShow>
    }
}

/// Component to delete a sphere category
#[component]
pub fn DeleteCategoryButton(
    category_name: String,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    let category_name = StoredValue::new(category_name);
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            <ActionForm
                action=sphere_state.delete_sphere_category_action
                attr:class="h-fit flex justify-center"
            >
                <input
                    name="sphere_name"
                    class="hidden"
                    value=sphere_state.sphere_name
                />
                <input
                    name="category_name"
                    class="hidden"
                    value=category_name.get_value()
                />
                <button class="p-1 rounded-xs bg-error hover:bg-error/75 active:scale-90 transition duration-250">
                    <CrossIcon/>
                </button>
            </ActionForm>
        </AuthorizedShow>
    }
}

pub fn get_sphere_category_header_map(
    sphere_category_load: Result<Vec<SphereCategory>, ServerFnError<AppError>>
) -> HashMap<i64, SphereCategoryHeader> {
    let mut sphere_category_map = HashMap::<i64, SphereCategoryHeader>::new();
    if let Ok(sphere_category_vec) = sphere_category_load {
        for sphere_category in sphere_category_vec {
            sphere_category_map.insert(sphere_category.category_id, sphere_category.clone().into());
        }
    }
    sphere_category_map
}

#[cfg(test)]
mod tests {
    use leptos::prelude::ServerFnError;
    use sharesphere_utils::colors::Color;
    use sharesphere_utils::errors::AppError;

    use crate::sphere_category::{get_sphere_category_header_map, SphereCategory};

    #[test]
    fn test_get_sphere_category_header_map() {
        let category_1 = SphereCategory {
            category_id: 0,
            sphere_id: 0,
            sphere_name: "a".to_string(),
            category_name: "a".to_string(),
            category_color: Color::None,
            description: "".to_string(),
            is_active: false,
            creator_id: 0,
            timestamp: Default::default(),
            delete_timestamp: None,
        };
        let category_2 = SphereCategory {
            category_id: 1,
            sphere_id: 1,
            sphere_name: "b".to_string(),
            category_name: "b".to_string(),
            category_color: Color::None,
            description: "".to_string(),
            is_active: false,
            creator_id: 0,
            timestamp: Default::default(),
            delete_timestamp: None,
        };
        let sphere_category_vec = vec![
            category_1.clone(),
            category_2.clone(),
        ];
        
        let category_map = get_sphere_category_header_map(Ok(sphere_category_vec));
        
        assert_eq!(category_map.len(), 2);
        
        assert_eq!(category_map.get(&category_1.category_id), Some(&category_1.into()));
        assert_eq!(category_map.get(&category_2.category_id), Some(&category_2.into()));
        
        let empty_category_map = get_sphere_category_header_map(Err(ServerFnError::<AppError>::Request(String::from("test"))));
        
        assert_eq!(empty_category_map.len(), 0);
    }
}