use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use leptos_use::{signal_debounced};

use sharesphere_utils::constants::{DELETED_MESSAGE};
use sharesphere_utils::editor::{TextareaData};
use sharesphere_utils::embed::{Embed, EmbedType, LinkType};
use sharesphere_utils::icons::{EditIcon};
use sharesphere_utils::routes::{get_post_id_memo, CREATE_POST_SPHERE_QUERY_PARAM};
use sharesphere_utils::unpack::{ActionError, SuspenseUnpack, TransitionUnpack};
use sharesphere_utils::widget::{ContentBody, DotMenu, ModalDialog, ModalFormButtons, ModeratorWidget, ScoreIndicator, TimeSinceEditWidget, TimeSinceWidget};

use sharesphere_auth::auth_widget::{AuthorWidget, DeleteButton};
use sharesphere_core::comment::{CommentWithChildren, COMMENT_BATCH_SIZE};
use sharesphere_core::moderation::{Content, ModeratedBody};
use sharesphere_core::post::{get_post_inherited_attributes, get_post_with_info_by_id, CreatePost, Post, PostBadgeList, PostForm, PostWithInfo};
use sharesphere_core::search::get_matching_sphere_header_vec;
use sharesphere_core::sphere::{SphereHeader};
use sharesphere_core::sphere_category::{get_sphere_category_vec};
use sharesphere_core::state::{GlobalState, SphereState};

use crate::comment::{CommentButtonWithCount, CommentSection};
use crate::moderation::{ModeratePostButton, ModerationInfoButton};
use crate::ranking::{VotePanel};

/// Component to display a content
#[component]
pub fn Post() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();
    let params = use_params_map();
    let post_id = get_post_id_memo(params);

    let post_resource = Resource::new(
        move || (
            post_id.get(),
            state.edit_post_action.version().get(),
            state.delete_post_action.version().get(),
            sphere_state.moderate_post_action.version().get()
        ),
        move |(post_id, _, _, _)| {
            log::debug!("Load data for post: {post_id}");
            get_post_with_info_by_id(post_id)
        },
    );

    let comment_vec = RwSignal::new(Vec::<CommentWithChildren>::with_capacity(
        COMMENT_BATCH_SIZE as usize,
    ));
    let is_loading = RwSignal::new(false);
    let additional_load_count = RwSignal::new(0);
    let container_ref = NodeRef::<html::Div>::new();

    view! {
        <div
            class="grow flex flex-col content-start gap-1 overflow-y-auto px-0.5"
            on:scroll=move |_| match container_ref.get() {
                Some(node_ref) => {
                    if !is_loading.get_untracked() && node_ref.scroll_top() + node_ref.offset_height() >= node_ref.scroll_height() {
                        additional_load_count.update(|value| *value += 1);
                    }
                },
                None => log::error!("Post container 'div' node failed to load."),
            }
            node_ref=container_ref
        >
            <TransitionUnpack resource=post_resource let:post_with_info>
                <div class="card">
                    <div class="card-body">
                        <div class="flex flex-col gap-1 2xl:gap-2">
                            <h2 class="card-title">
                            { match post_with_info.post.is_active() {
                                true => post_with_info.post.title.clone(),
                                false => DELETED_MESSAGE.to_string()
                            }}
                            </h2>
                            <PostBody post=&post_with_info.post/>
                            <Embed link=post_with_info.post.link.clone()/>
                            <PostBadgeList
                                sphere_header=None
                                sphere_category=post_with_info.sphere_category.clone()
                                is_spoiler=post_with_info.post.is_spoiler
                                is_nsfw=post_with_info.post.is_nsfw
                                is_pinned=post_with_info.post.is_pinned
                            />
                            <PostWidgetBar post=post_with_info comment_vec/>
                        </div>
                    </div>
                </div>
            </TransitionUnpack>
            <CommentSection post_id comment_vec is_loading additional_load_count/>
        </div>
    }.into_any()
}

/// Displays the body of a post
#[component]
pub fn PostBody<'a>(post: &'a Post) -> impl IntoView {

    view! {
        <div class="pb-2">
        {
            match (&post.delete_timestamp, &post.moderator_message, &post.infringed_rule_title) {
                (Some(_), _, _) => view! {
                    <ContentBody
                        body=DELETED_MESSAGE.to_string()
                        is_markdown=false
                    />
                }.into_any(),
                (None, Some(moderator_message), Some(infringed_rule_title)) => view! {
                    <ModeratedBody
                        infringed_rule_title=infringed_rule_title.clone()
                        moderator_message=moderator_message.clone()
                    />
                }.into_any(),
                _ => view! {
                    <ContentBody
                        body=post.body.clone()
                        is_markdown=post.markdown_body.is_some()
                    />
                }.into_any(),
            }
        }
        </div>
    }.into_any()
}

/// Component to encapsulate the widgets associated with each post
#[component]
fn PostWidgetBar<'a>(
    post: &'a PostWithInfo,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
) -> impl IntoView {
    let post_id = post.post.post_id;
    let author_id = post.post.creator_id;
    let is_active = post.post.is_active();
    let stored_post = StoredValue::new(post.post.clone());
    view! {
        <div class="flex gap-1 items-center">
            { match is_active {
                true => Either::Left(view! {
                    <VotePanel
                        post_id=post.post.post_id
                        comment_id=None
                        score=post.post.score
                        vote=post.vote.clone()
                    />
                }),
                false => Either::Right(view! {
                    <ScoreIndicator score=post.post.score/>
                }),
            }}
            <CommentButtonWithCount post_id comment_vec count=post.post.num_comments/>
            {
                is_active.then_some(view! {
                    <AuthorWidget author=post.post.creator_name.clone() is_moderator=post.post.is_creator_moderator/>
                })
            }
            <ModeratorWidget moderator=post.post.moderator_name.clone()/>
            <TimeSinceWidget timestamp=post.post.create_timestamp/>
            <TimeSinceEditWidget edit_timestamp=post.post.edit_timestamp/>
            <DotMenu>
                { is_active.then_some(view! {
                    <EditPostButton author_id post=stored_post/>
                    <ModeratePostButton post_id/>
                    <DeletePostButton post_id author_id/>

                })}
                <ModerationInfoButton content=Content::Post(stored_post.get_value())/>
            </DotMenu>
        </div>
    }
}

/// Component to edit a post
#[component]
pub fn EditPostButton(
    post: StoredValue<Post>,
    author_id: i64
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let show_dialog = RwSignal::new(false);
    let show_button = move || match &(*state.user.read()) {
        Some(Ok(Some(user))) => user.user_id == author_id,
        _ => false,
    };
    let edit_button_class = move || match show_dialog.get() {
        true => "button-rounded-primary",
        false => "button-rounded-neutral",
    };
    view! {
        <Show when=show_button>
            <div>
                <button
                    class=edit_button_class
                    aria-expanded=move || show_dialog.get().to_string()
                    aria-haspopup="dialog"
                    on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
                >
                    <EditIcon/>
                </button>
                <EditPostDialog
                    post=post.get_value()
                    show_dialog
                />
            </div>
        </Show>
    }
}

/// Component to delete a post
#[component]
pub fn DeletePostButton(
    post_id: i64,
    author_id: i64,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <DeleteButton
            title="Delete Post"
            id=post_id
            id_name="post_id"
            author_id
            delete_action=state.delete_post_action
        />
    }
}

/// Component to create a new post
#[component]
pub fn CreatePost() -> impl IntoView {
    let create_post_action = ServerAction::<CreatePost>::new();

    let query = use_query_map();
    let sphere_query = move || {
        query.read_untracked().get(CREATE_POST_SPHERE_QUERY_PARAM).unwrap_or_default()
    };

    let is_sphere_selected = RwSignal::new(false);
    let is_sphere_nsfw = RwSignal::new(false);
    let sphere_name_input = RwSignal::new(sphere_query());
    let sphere_name_debounced: Signal<String> = signal_debounced(sphere_name_input, 250.0);

    let title_input = RwSignal::new(String::default());
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_data = TextareaData {
        content: RwSignal::new(String::new()),
        textarea_ref,
    };
    let link_input = RwSignal::new(String::default());
    let embed_type_input = RwSignal::new(EmbedType::None);

    let matching_spheres_resource = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_prefix| get_matching_sphere_header_vec(sphere_prefix),
    );

    let category_vec_resource = Resource::new(
        move || sphere_name_debounced.get(),
        move |sphere_name| get_sphere_category_vec(sphere_name)
    );

    view! {
        <div class="w-4/5 2xl:w-2/5 p-2 mx-auto flex flex-col gap-2 overflow-auto">
            <ActionForm action=create_post_action>
                <div class="flex flex-col gap-2 w-full">
                    <h2 class="py-4 text-4xl text-center">"Share a post!"</h2>
                    <div class="dropdown dropdown-end">
                        <input
                            tabindex="0"
                            type="text"
                            name="sphere"
                            placeholder="Sphere"
                            autocomplete="off"
                            class="input input-primary w-full"
                            on:input=move |ev| {
                                sphere_name_input.set(event_target_value(&ev).to_lowercase());
                            }
                            prop:value=sphere_name_input
                        />
                        <ul tabindex="0" class="dropdown-content z-1 menu p-2 shadow-sm bg-base-200 rounded-xs w-full">
                            <TransitionUnpack resource=matching_spheres_resource let:sphere_header_vec>
                            {
                                match sphere_header_vec.first() {
                                    Some(header) if header.sphere_name == sphere_name_input.get_untracked() => {
                                        is_sphere_nsfw.set(header.is_nsfw);
                                        is_sphere_selected.set(true);
                                    },
                                    _ => {
                                        is_sphere_selected.set(false);
                                        is_sphere_nsfw.set(false);
                                    }
                                };
                                sphere_header_vec.clone().into_iter().map(|sphere_header| {
                                    let sphere_name = sphere_header.sphere_name.clone();
                                    view! {
                                        <li>
                                            <button
                                                type="button"
                                                on:click=move |_| sphere_name_input.set(sphere_name.clone())
                                            >
                                                <SphereHeader sphere_header/>
                                            </button>
                                        </li>
                                    }
                                }).collect_view()
                            }
                            </TransitionUnpack>
                        </ul>
                    </div>
                    <PostForm
                        title_input
                        body_data
                        embed_type_input
                        link_input
                        sphere_name=sphere_name_input
                        is_parent_spoiler=false
                        is_parent_nsfw=is_sphere_nsfw
                        category_vec_resource
                    />
                    <button type="submit" class="button-secondary" disabled=move || {
                        !is_sphere_selected.get() ||
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

/// Dialog to edit a post
#[component]
pub fn EditPostDialog(
    post: Post,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    let post = StoredValue::new(post);
    view! {
        <ModalDialog
            class="w-full flex justify-center"
            show_dialog
        >
            <EditPostForm
                post
                show_form=show_dialog
            />
        </ModalDialog>
    }
}

/// Form to edit a post
#[component]
pub fn EditPostForm(
    post: StoredValue<Post>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let sphere_state = expect_context::<SphereState>();

    let (post_id, title, link_type, link_url) = post.with_value(|post| (
        post.post_id,
        post.title.clone(),
        post.link.link_type,
        post.link.link_url.clone(),
    ));
    let title_input = RwSignal::new(title);
    let textarea_ref = NodeRef::<html::Textarea>::new();
    let body_data = TextareaData {
        content: RwSignal::new(String::new()),
        textarea_ref,
    };
    let embed_type_input = RwSignal::new(match link_type {
        LinkType::None => EmbedType::None,
        LinkType::Link => EmbedType::Link,
        _ => EmbedType::Embed,
    });
    let link_input = RwSignal::new(link_url.unwrap_or_default());
    let disable_publish = Signal::derive(move || {
        title_input.read().is_empty() ||
        (
            body_data.content.read().is_empty() &&
            embed_type_input.read() == EmbedType::None
        ) || (
            embed_type_input.read() != EmbedType::None &&
            link_input.read().is_empty()
        )
    });

    let inherited_attributes_resource = Resource::new(
        move || (),
        move |_| get_post_inherited_attributes(post_id)
    );

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-xs flex flex-col gap-3 w-4/5 2xl:w-2/5">
            <div class="text-center font-bold text-2xl">"Edit your post"</div>
            <ActionForm action=state.edit_post_action>
                <div class="flex flex-col gap-3 w-full">
                    <input
                        type="text"
                        name="post_id"
                        class="hidden"
                        value=post_id
                    />
                    <SuspenseUnpack resource=inherited_attributes_resource let:inherited_post_attr>
                        <PostForm
                            title_input
                            body_data
                            embed_type_input
                            link_input
                            sphere_name=sphere_state.sphere_name
                            is_parent_spoiler=inherited_post_attr.is_spoiler
                            is_parent_nsfw=inherited_post_attr.is_nsfw
                            category_vec_resource=sphere_state.sphere_categories_resource
                            current_post=Some(post)
                        />
                    </SuspenseUnpack>
                    <ModalFormButtons
                        disable_publish
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError action=state.edit_post_action.into()/>
        </div>
    }
}
