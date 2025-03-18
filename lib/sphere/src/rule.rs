use leptos::form::ActionForm;
use leptos::html;
use leptos::prelude::*;
use leptos_use::use_textarea_autosize;

use sharesphere_utils::editor::{FormTextEditor, TextareaData};
use sharesphere_utils::icons::{CrossIcon, EditIcon, PlusIcon};
use sharesphere_utils::unpack::{TransitionUnpack};
use sharesphere_utils::widget::{ModalDialog, ModalFormButtons};

use sharesphere_auth::role::{AuthorizedShow, PermissionLevel};

use sharesphere_core::rule::Rule;
use sharesphere_core::state::SphereState;

/// Component to manage sphere rules
#[component]
pub fn SphereRulesPanel() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    view! {
        // TODO add overflow-y-auto max-h-full?
        <div class="shrink-0 flex flex-col gap-1 content-center w-full h-fit bg-base-200 p-2 rounded-sm">
            <div class="text-xl text-center">"Rules"</div>
            <div class="flex flex-col gap-1">
                <div class="border-b border-base-content/20 pl-1">
                    <div class="w-5/6 flex gap-1">
                        <div class="w-1/12 py-2 font-bold">"N°"</div>
                        <div class="w-5/12 py-2 font-bold">"Title"</div>
                        <div class="w-6/12 py-2 font-bold">"Description"</div>
                    </div>
                </div>
                <TransitionUnpack resource=sphere_state.sphere_rules_resource let:sphere_rule_vec>
                {
                    let sphere_rule_vec = sphere_rule_vec.clone();
                    view! {
                        <For
                            each=move || sphere_rule_vec.clone().into_iter()
                            key=|rule| rule.rule_id
                            children=move |rule| {
                                let rule = StoredValue::new(rule);
                                let show_edit_form = RwSignal::new(false);
                                view! {
                                    <div class="flex gap-1 justify-between rounded-sm pl-1">
                                        <div class="w-5/6 flex gap-1">
                                            <div class="w-1/12 select-none">{rule.get_value().priority}</div>
                                            <div class="w-5/12 select-none">{rule.get_value().title}</div>
                                            <div class="w-6/12 select-none">{rule.get_value().description}</div>
                                        </div>
                                        <div class="flex gap-1 justify-end">
                                            <button
                                                class="h-fit p-1 text-sm bg-secondary rounded-xs hover:bg-secondary/75 active:scale-90 transition duration-250"
                                                on:click=move |_| show_edit_form.update(|value| *value = !*value)
                                            >
                                                <EditIcon/>
                                            </button>
                                            <DeleteRuleButton rule/>
                                        </div>
                                    </div>
                                    <ModalDialog
                                        class="w-full max-w-xl"
                                        show_dialog=show_edit_form
                                    >
                                        <EditRuleForm rule show_form=show_edit_form/>
                                    </ModalDialog>
                                }
                            }
                        />
                    }
                }
                </TransitionUnpack>
            </div>
            <CreateRuleForm/>
        </div>
    }
}

/// Component to delete a sphere rule
#[component]
pub fn DeleteRuleButton(
    rule: StoredValue<Rule>
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let sphere_name = sphere_state.sphere_name;
    view! {
        <AuthorizedShow sphere_name permission_level=PermissionLevel::Manage>
            <ActionForm
                action=sphere_state.remove_rule_action
                attr:class="h-fit flex justify-center"
            >
                <input
                    name="sphere_name"
                    class="hidden"
                    value=sphere_state.sphere_name
                />
                <input
                    name="priority"
                    class="hidden"
                    value=rule.with_value(|rule| rule.priority)
                />
                <button class="p-1 rounded-xs bg-error hover:bg-error/75 active:scale-90 transition duration-250">
                    <CrossIcon/>
                </button>
            </ActionForm>
        </AuthorizedShow>
    }
}

/// Component to edit a sphere rule
#[component]
pub fn EditRuleForm(
    rule: StoredValue<Rule>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let rule_priority = rule.with_value(|rule| rule.priority);
    let priority = RwSignal::new(rule_priority.to_string());
    let title_ref = NodeRef::<html::Textarea>::new();
    let title_autosize = use_textarea_autosize(title_ref);
    let title_data = TextareaData {
        content: title_autosize.content,
        set_content: title_autosize.set_content,
        textarea_ref: title_ref,
    };
    let description_ref = NodeRef::<html::Textarea>::new();
    let desc_autosize = use_textarea_autosize(description_ref);
    let description_data = TextareaData {
        content: desc_autosize.content,
        set_content: desc_autosize.set_content,
        textarea_ref: description_ref,
    };
    let invalid_inputs = Signal::derive(move || {
        priority.read().is_empty() || title_autosize.content.read().is_empty() || description_data.content.read().is_empty()
    });

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Edit a rule"</div>
            <ActionForm action=sphere_state.update_rule_action>
                <input
                    name="sphere_name"
                    class="hidden"
                    value=sphere_state.sphere_name
                />
                <input
                    name="current_priority"
                    class="hidden"
                    value=rule_priority
                />
                <div class="flex flex-col gap-3 w-full">
                    <RuleInputs priority title_data description_data/>
                    <ModalFormButtons
                        disable_publish=invalid_inputs
                        show_form
                    />
                </div>
            </ActionForm>
        </div>
    }
}

/// Component to create a sphere rule
#[component]
pub fn CreateRuleForm() -> impl IntoView {
    let sphere_state = expect_context::<SphereState>();
    let show_dialog = RwSignal::new(false);
    let priority = RwSignal::new(String::default());
    let title_ref = NodeRef::<html::Textarea>::new();
    let title_autosize = use_textarea_autosize(title_ref);
    let title_data = TextareaData {
        content: title_autosize.content,
        set_content: title_autosize.set_content,
        textarea_ref: title_ref,
    };
    let description_ref = NodeRef::<html::Textarea>::new();
    let desc_autosize = use_textarea_autosize(description_ref);
    let description_data = TextareaData {
        content: desc_autosize.content,
        set_content: desc_autosize.set_content,
        textarea_ref: description_ref,
    };
    let invalid_inputs = Signal::derive(move || {
        priority.read().is_empty() || title_autosize.content.read().is_empty() || description_data.content.read().is_empty()
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
            <div class="text-center font-bold text-2xl">"Add a rule"</div>
                <ActionForm
                    action=sphere_state.add_rule_action
                    on:submit=move |_| show_dialog.set(false)
                >
                    <input
                        name="sphere_name"
                        class="hidden"
                        value=sphere_state.sphere_name
                    />
                    <div class="flex flex-col gap-3 w-full">
                        <RuleInputs priority title_data description_data/>
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

/// Components with inputs to create or edit a rule
#[component]
pub fn RuleInputs(
    priority: RwSignal<String>,
    title_data: TextareaData,
    description_data: TextareaData,
) -> impl IntoView {
    view! {
        <div class="flex gap-1 content-center">
            <input
                tabindex="0"
                type="number"
                name="priority"
                placeholder="N°"
                autocomplete="off"
                class="input input-primary no-spinner px-1 w-1/12"
                value=priority
                on:input=move |ev| priority.set(event_target_value(&ev))
            />
            <FormTextEditor
                name="title"
                placeholder="Title"
                data=title_data
                class="w-5/12"
            />
            <FormTextEditor
                name="description"
                placeholder="Description"
                data=description_data
                class="w-6/12"
            />
        </div>
    }
}
