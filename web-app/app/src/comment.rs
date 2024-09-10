use leptos::either::Either;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::app::GlobalState;
#[cfg(feature = "ssr")]
use crate::auth::ssr::check_user;
use crate::auth::LoginGuardButton;
use crate::constants::{BEST_STR, RECENT_STR};
use crate::content::{Content, ContentBody};
#[cfg(feature = "ssr")]
use crate::editor::get_styled_html_from_markdown;
use crate::editor::FormMarkdownEditor;
use crate::icons::{CommentIcon, EditIcon};
use crate::moderation::{ModerateCommentButton, ModeratedBody, ModerationInfoButton};
use crate::navigation_bar::get_current_path;
#[cfg(feature = "ssr")]
use crate::ranking::{ssr::vote_on_content, VoteValue};
use crate::ranking::{SortType, Vote, VotePanel};
use crate::widget::{ActionError, AuthorWidget, MinimizeMaximizeWidget, ModalDialog, ModalFormButtons, ModeratorWidget, TimeSinceEditWidget, TimeSinceWidget};
#[cfg(feature = "ssr")]
use crate::{app::ssr::get_db_pool, auth::get_user};

pub const COMMENT_BATCH_SIZE: i64 = 50;
const DEPTH_TO_COLOR_MAPPING_SIZE: usize = 6;
const DEPTH_TO_COLOR_MAPPING: [&str; DEPTH_TO_COLOR_MAPPING_SIZE] = [
    "bg-blue-500",
    "bg-green-500",
    "bg-yellow-500",
    "bg-orange-500",
    "bg-red-500",
    "bg-violet-500",
];

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Comment {
    pub comment_id: i64,
    pub body: String,
    pub markdown_body: Option<String>,
    pub is_edited: bool,
    pub moderator_message: Option<String>,
    pub infringed_rule_id: Option<i64>,
    pub infringed_rule_title: Option<String>,
    pub parent_id: Option<i64>,
    pub post_id: i64,
    pub creator_id: i64,
    pub creator_name: String,
    pub moderator_id: Option<i64>,
    pub moderator_name: Option<String>,
    pub score: i32,
    pub score_minus: i32,
    pub create_timestamp: chrono::DateTime<chrono::Utc>,
    pub edit_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CommentWithChildren {
    pub comment: Comment,
    pub vote: Option<Vote>,
    pub child_comments: Vec<CommentWithChildren>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CommentSortType {
    Best,
    Recent,
}

impl fmt::Display for CommentSortType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sort_type_name = match self {
            CommentSortType::Best => BEST_STR,
            CommentSortType::Recent => RECENT_STR,
        };
        write!(f, "{sort_type_name}")
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::PgPool;

    use crate::constants::{BEST_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN};
    use crate::errors::AppError;
    use crate::forum::Forum;
    use crate::post::ssr::get_post_forum;
    use crate::ranking::VoteValue;
    use crate::user::User;

    use super::*;

    #[derive(
        Clone, Debug, PartialEq, Eq, sqlx::FromRow, Ord, PartialOrd, Serialize, Deserialize,
    )]
    pub struct CommentWithVote {
        #[sqlx(flatten)]
        pub comment: Comment,
        pub vote_id: Option<i64>,
        pub vote_post_id: Option<i64>,
        pub vote_comment_id: Option<Option<i64>>,
        pub vote_user_id: Option<i64>,
        pub value: Option<i16>,
        pub vote_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl CommentWithVote {
        pub fn into_comment_with_children(self) -> CommentWithChildren {
            let comment_vote = if self.vote_id.is_some() {
                Some(Vote {
                    vote_id: self.vote_id.unwrap(),
                    user_id: self.vote_user_id.unwrap(),
                    comment_id: self.vote_comment_id.unwrap(),
                    post_id: self.vote_post_id.unwrap(),
                    value: VoteValue::from(self.value.unwrap()),
                    timestamp: self.vote_timestamp.unwrap(),
                })
            } else {
                None
            };

            CommentWithChildren {
                comment: self.comment,
                vote: comment_vote,
                child_comments: Vec::<CommentWithChildren>::new(),
            }
        }
    }

    impl CommentSortType {
        pub fn to_order_by_code(self) -> &'static str {
            match self {
                CommentSortType::Best => BEST_ORDER_BY_COLUMN,
                CommentSortType::Recent => RECENT_ORDER_BY_COLUMN,
            }
        }
    }

    pub async fn get_comment_by_id(
        comment_id: i64,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let comment = sqlx::query_as!(
            Comment,
            "SELECT * FROM comments
            WHERE comment_id = $1",
            comment_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(comment)
    }
    
    pub async fn get_comment_forum(
        comment_id: i64,
        db_pool: &PgPool,
    ) -> Result<Forum, AppError> {
        let forum = sqlx::query_as!(
            Forum,
            "SELECT f.*
            FROM forums f
            JOIN posts p on p.forum_id = f.forum_id
            JOIN comments c on c.post_id = p.post_id
            WHERE c.comment_id = $1",
            comment_id
        )
            .fetch_one(db_pool)
            .await?;

        Ok(forum)
    }

    pub async fn get_post_comment_tree(
        post_id: i64,
        sort_type: SortType,
        user_id: Option<i64>,
        limit: i64,
        offset: i64,
        db_pool: &PgPool,
    ) -> Result<Vec<CommentWithChildren>, AppError> {
        if post_id < 1 {
            return Err(AppError::new("Invalid post id."));
        }

        let sort_column = sort_type.to_order_by_code();

        let comment_with_vote_vec = sqlx::query_as::<_, CommentWithVote>(
            format!(
                "WITH RECURSIVE comment_tree AS (
                (
                    SELECT c.*,
                           v.vote_id,
                           v.user_id as vote_user_id,
                           v.post_id as vote_post_id,
                           v.comment_id as vote_comment_id,
                           v.value,
                           v.timestamp as vote_timestamp,
                           1 AS depth,
                           ARRAY[(c.{sort_column}, c.comment_id)] AS path
                    FROM comments c
                    LEFT JOIN votes v
                    ON v.comment_id = c.comment_id AND
                       v.user_id = $1
                    WHERE
                        c.post_id = $2 AND
                        c.parent_id IS NULL
                    ORDER BY c.{sort_column} DESC
                    LIMIT $3
                    OFFSET $4
                )
                UNION ALL
                (
                    SELECT n.*,
                           vr.vote_id,
                           vr.user_id as vote_user_id,
                           vr.post_id as vote_post_id,
                           vr.comment_id as vote_comment_id,
                           vr.value,
                           vr.timestamp as vote_timestamp,
                           r.depth + 1,
                           r.path || (n.{sort_column}, n.comment_id)
                    FROM comment_tree r
                    JOIN comments n ON n.parent_id = r.comment_id
                    LEFT JOIN votes vr
                    ON vr.comment_id = n.comment_id AND
                       vr.user_id = $1
                )
            )
            SELECT * FROM comment_tree
            ORDER BY path DESC"
            )
                .as_str(),
        )
            .bind(user_id)
            .bind(post_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        let mut comment_tree = Vec::<CommentWithChildren>::new();
        let mut stack = Vec::<(i64, Vec<CommentWithChildren>)>::new();
        for comment_with_vote in comment_with_vote_vec {
            let mut current = comment_with_vote.into_comment_with_children();

            if let Some((top_parent_id, child_comments)) = stack.last_mut() {
                if *top_parent_id == current.comment.comment_id {
                    // child comments at the top of the stack belong to the current comment, add them
                    current.child_comments.append(child_comments);
                    stack.pop();
                }
            }

            // if the current element has a parent, add it to the stack. Otherwise, add it to the comment tree as a root element.
            if let Some(parent_id) = current.comment.parent_id {
                if let Some((top_parent_id, top_child_comments)) = stack.last_mut() {
                    if parent_id == *top_parent_id {
                        // same parent id as the top of the stack, add it
                        top_child_comments.push(current);
                    } else {
                        // different parent id as the top of the stack, add it as a new element on the stack
                        stack.push((parent_id, Vec::from([current])));
                    }
                } else {
                    // no element on the stack, add the current comment as a new element
                    stack.push((parent_id, Vec::from([current])));
                }
            } else {
                comment_tree.push(current);
            }
        }

        Ok(comment_tree)
    }

    pub async fn create_comment(
        post_id: i64,
        parent_comment_id: Option<i64>,
        comment: &str,
        markdown_comment: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let forum = get_post_forum(post_id, &db_pool).await?;
        user.check_can_publish_on_forum(&forum.forum_name)?;
        if comment.is_empty() {
            return Err(AppError::new("Cannot create empty comment."));
        }

        let comment = sqlx::query_as!(
            Comment,
            "INSERT INTO comments (body, markdown_body, parent_id, post_id, creator_id, creator_name) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
            comment,
            markdown_comment,
            parent_comment_id,
            post_id,
            user.user_id,
            user.username,
        )
            .fetch_one(db_pool)
            .await?;

        // TODO increase post comment count

        Ok(comment)
    }

    pub async fn update_comment(
        comment_id: i64,
        comment_body: &str,
        comment_markdown_body: Option<&str>,
        user: &User,
        db_pool: &PgPool,
    ) -> Result<Comment, AppError> {
        let comment = sqlx::query_as!(
            Comment,
            "UPDATE comments SET
                body = $1,
                markdown_body = $2,
                edit_timestamp = CURRENT_TIMESTAMP
            WHERE
                comment_id = $3 AND
                creator_id = $4
            RETURNING *",
            comment_body,
            comment_markdown_body,
            comment_id,
            user.user_id,
        )
        .fetch_one(db_pool)
        .await?;

        Ok(comment)
    }

    #[cfg(test)]
    mod tests {
        use crate::comment::ssr::CommentWithVote;
        use crate::comment::{Comment, CommentSortType};
        use crate::constants::{BEST_ORDER_BY_COLUMN, RECENT_ORDER_BY_COLUMN};
        use crate::ranking::VoteValue;
        use crate::user::User;

        #[test]
        fn test_comment_join_vote_into_comment_with_children() {
            let user = User::default();
            let mut comment = Comment::default();
            comment.creator_id = user.user_id;

            let comment_without_vote = CommentWithVote {
                comment: comment.clone(),
                vote_id: None,
                vote_post_id: None,
                vote_comment_id: None,
                vote_user_id: None,
                value: None,
                vote_timestamp: None,
            };
            let comment_without_vote = comment_without_vote.into_comment_with_children();
            assert_eq!(comment_without_vote.comment, comment);
            assert_eq!(comment_without_vote.vote, None);
            assert_eq!(comment_without_vote.child_comments.is_empty(), true);

            let comment_with_vote = CommentWithVote {
                comment: comment.clone(),
                vote_id: Some(0),
                vote_post_id: Some(comment.post_id),
                vote_comment_id: Some(Some(comment.comment_id)),
                vote_user_id: Some(user.user_id),
                value: Some(1),
                vote_timestamp: Some(comment.create_timestamp),
            };
            let comment_with_vote = comment_with_vote.into_comment_with_children();
            let user_vote = comment_with_vote.vote.expect("CommentWithChildren should contain vote.");
            assert_eq!(comment_with_vote.comment, comment);
            assert_eq!(user_vote.user_id, user.user_id);
            assert_eq!(user_vote.comment_id, Some(comment.comment_id));
            assert_eq!(user_vote.value, VoteValue::Up);
            assert_eq!(user_vote.comment_id, Some(comment.comment_id));
            assert_eq!(comment_with_vote.child_comments.is_empty(), true);
        }

        #[test]
        fn test_comment_sort_type_to_order_by_code() {
            assert_eq!(CommentSortType::Best.to_order_by_code(), BEST_ORDER_BY_COLUMN);
            assert_eq!(CommentSortType::Recent.to_order_by_code(), RECENT_ORDER_BY_COLUMN);
        }
    }
}

#[server]
pub async fn get_post_comment_tree(
    post_id: i64,
    sort_type: SortType,
    num_already_loaded: usize,
) -> Result<Vec<CommentWithChildren>, ServerFnError> {
    let user_id = match get_user().await {
        Ok(Some(user)) => Some(user.user_id),
        _ => None,
    };
    let db_pool = get_db_pool()?;
    let comment_tree = ssr::get_post_comment_tree(
        post_id,
        sort_type,
        user_id,
        COMMENT_BATCH_SIZE,
        num_already_loaded as i64,
        &db_pool,
    )
    .await?;

    Ok(comment_tree)
}

#[server]
pub async fn create_comment(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment: String,
    is_markdown: bool,
) -> Result<CommentWithChildren, ServerFnError> {
    log::trace!("Create comment for post {post_id}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = match is_markdown {
        true => (
            get_styled_html_from_markdown(comment.clone()).await?,
            Some(comment.as_str()),
        ),
        false => (comment, None),
    };

    let mut comment = ssr::create_comment(
        post_id,
        parent_comment_id,
        comment.as_str(),
        markdown_comment,
        &user,
        &db_pool,
    )
    .await?;

    // TODO: move in create comment?
    let vote = vote_on_content(
        VoteValue::Up,
        comment.post_id,
        Some(comment.comment_id),
        None,
        &user,
        &db_pool,
    )
    .await?;
    comment.score = 1;

    Ok(CommentWithChildren {
        comment,
        vote,
        child_comments: Vec::<CommentWithChildren>::default(),
    })
}

#[server]
pub async fn edit_comment(
    comment_id: i64,
    comment: String,
    is_markdown: bool,
) -> Result<Comment, ServerFnError> {
    log::trace!("Edit comment {comment_id}");
    let user = check_user()?;
    let db_pool = get_db_pool()?;

    let (comment, markdown_comment) = match is_markdown {
        true => (
            get_styled_html_from_markdown(comment.clone()).await?,
            Some(comment.as_str()),
        ),
        false => (comment, None),
    };

    let comment = ssr::update_comment(
        comment_id,
        comment.as_str(),
        markdown_comment,
        &user,
        &db_pool,
    )
    .await?;

    Ok(comment)
}

/// Comment section component
#[component]
pub fn CommentSection(comment_vec: RwSignal<Vec<CommentWithChildren>>) -> impl IntoView {
    view! {
        <div class="flex flex-col h-fit">
            <For
                // a function that returns the items we're iterating over; a signal is fine
                each= move || comment_vec.get().into_iter().enumerate()
                // a unique key for each item as a reference
                key=|(_index, comment)| comment.comment.comment_id
                // renders each item to a view
                children=move |(index, comment_with_children)| {
                    view! {
                        <CommentBox
                            comment_with_children
                            depth=0
                            ranking=index
                        />
                    }
                }
            />
        </div>
    }
}

/// Comment box component
#[component]
pub fn CommentBox(
    comment_with_children: CommentWithChildren,
    depth: usize,
    ranking: usize,
) -> impl IntoView {
    let comment = RwSignal::new(comment_with_children.comment);
    let child_comments = RwSignal::new(comment_with_children.child_comments);
    let maximize = RwSignal::new(true);
    let sidebar_css = move || {
        if *maximize.read() {
            "p-0.5 rounded hover:bg-base-content/20 flex flex-col justify-start items-center gap-1"
        } else {
            "p-0.5 rounded hover:bg-base-content/20 flex flex-col justify-center items-center"
        }
    };
    let color_bar_css = format!(
        "{} rounded-full h-full w-1 ",
        DEPTH_TO_COLOR_MAPPING[(depth + ranking) % DEPTH_TO_COLOR_MAPPING.len()]
    );

    view! {
        <div class="flex gap-1 py-1">
            <div
                class=sidebar_css
                on:click=move |_| maximize.update(|value: &mut bool| *value = !*value)
            >
                <MinimizeMaximizeWidget is_maximized=maximize/>
                <Show
                    when=maximize
                >
                    <div class=color_bar_css.clone()/>
                </Show>
            </div>
            <div class="flex flex-col gap-1">
                <Show when=maximize>
                    <CommentBody comment/>
                </Show>
                <CommentWidgetBar
                    comment=comment
                    vote=comment_with_children.vote
                    child_comments
                />
                <div
                    class="flex flex-col"
                    class:hidden=move || !*maximize.read()
                >
                    <For
                        // a function that returns the items we're iterating over; a signal is fine
                        each= move || child_comments.get().into_iter().enumerate()
                        // a unique key for each item as a reference
                        key=|(_index, comment)| comment.comment.comment_id
                        // renders each item to a view
                        children=move |(index, comment_with_children)| {
                            view! {
                                <CommentBox
                                    comment_with_children
                                    depth=depth+1
                                    ranking=ranking+index
                                />
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }.into_any()
}

/// Displays the body of a comment
#[component]
pub fn CommentBody(
    #[prop(into)]
    comment: Signal<Comment>
) -> impl IntoView {
    view! {
        {
            move || comment.with(|comment| match (&comment.moderator_message, &comment.infringed_rule_title) {
                (Some(moderator_message), Some(infringed_rule_title)) => Either::Left(view! {
                    <ModeratedBody
                        infringed_rule_title=infringed_rule_title.clone()
                        moderator_message=moderator_message.clone()
                    />
                }),
                _ => Either::Right(view! {
                    <div class="pl-2">
                        <ContentBody 
                            body=comment.body.clone()
                            is_markdown=comment.markdown_body.is_some()
                        />
                    </div>
                }),
            })
        }
    }
}

/// Component to encapsulate the widgets associated with each comment
#[component]
fn CommentWidgetBar(
    comment: RwSignal<Comment>,
    vote: Option<Vote>,
    child_comments: RwSignal<Vec<CommentWithChildren>>,
) -> impl IntoView {
    let (comment_id, post_id, score, author_id, author) =
        comment.with_untracked(|comment| {
            (
                comment.comment_id,
                comment.post_id,
                comment.score,
                comment.creator_id,
                comment.creator_name.clone(),
            )
        });
    let timestamp = Signal::derive(move || comment.with(|comment| comment.create_timestamp));
    let edit_timestamp = Signal::derive(move || comment.with(|comment| comment.edit_timestamp));
    let moderator = Signal::derive(move || comment.with(|comment| comment.moderator_name.clone()));
    let content = Signal::derive(move || Content::Comment(comment.get()));
    view! {
        <div class="flex gap-1">
            <VotePanel
                post_id
                comment_id=Some(comment_id)
                score
                vote
            />
            <CommentButton
                post_id
                comment_vec=child_comments
                parent_comment_id=Some(comment_id)
            />
            <EditCommentButton
                comment_id
                author_id
                comment
            />
            <ModerateCommentButton
                comment_id
                comment
            />
            <ModerationInfoButton content/>
            <AuthorWidget author/>
            <ModeratorWidget moderator/>
            <TimeSinceWidget timestamp/>
            <TimeSinceEditWidget edit_timestamp/>
        </div>
    }
}

/// Component to open the comment form
#[component]
pub fn CommentButton(
    post_id: i64,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    #[prop(default = None)] parent_comment_id: Option<i64>,
) -> impl IntoView {
    let show_dialog = RwSignal::new(false);
    let redirect_path = RwSignal::new(String::new());
    let comment_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };

    view! {
        <div>
            //<LoginGuardButton
            //    login_button_class="btn btn-circle btn-ghost btn-sm"
            //    login_button_content=move || view! { <CommentIcon/> }
            //    redirect_path
            //    on:click=move |_| get_current_path(redirect_path)
            //    let:_user
            //>
            //    <button
            //        class=comment_button_class
            //        aria-expanded=move || show_dialog.get().to_string()
            //        aria-haspopup="dialog"
            //        on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
            //    >
            //        <CommentIcon/>
            //    </button>
            //</LoginGuardButton>
            <CommentDialog
                post_id
                parent_comment_id
                comment_vec
                show_dialog
            />
        </div>
    }
}

/// Dialog to publish a comment
#[component]
pub fn CommentDialog(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <CommentForm
                post_id
                parent_comment_id
                comment_vec
                show_form=show_dialog
            />
        </ModalDialog>
    }
}

/// Form to publish a comment
#[component]
pub fn CommentForm(
    post_id: i64,
    parent_comment_id: Option<i64>,
    comment_vec: RwSignal<Vec<CommentWithChildren>>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let comment = RwSignal::new(String::new());
    let is_comment_empty = move || comment.with(|comment: &String| comment.is_empty());

    let create_comment_action = ServerAction::<CreateComment>::new();

    let create_comment_result = create_comment_action.value();
    let has_error = move || create_comment_result.with(|val| matches!(val, Some(Err(_))));

    Effect::new(move |_| {
        if let Some(Ok(comment)) = create_comment_action.value().get() {
            comment_vec.update(|comment_vec| comment_vec.insert(0, comment));
            show_form.set(false);
        }
    });

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Share a comment"</div>
            <ActionForm action=create_comment_action>
                <div class="flex flex-col gap-3 w-full">
                    <input
                        type="text"
                        name="post_id"
                        class="hidden"
                        value=post_id
                    />
                    <input
                        type="text"
                        name="parent_comment_id"
                        class="hidden"
                        value=parent_comment_id
                    />
                    <FormMarkdownEditor
                        name="comment"
                        is_markdown_name="is_markdown"
                        placeholder="Your comment..."
                        content=comment
                    />
                    <ModalFormButtons
                        disable_publish=is_comment_empty
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError has_error/>
        </div>
    }
}

/// Component to open the edit comment form
#[component]
pub fn EditCommentButton(
    comment_id: i64,
    author_id: i64,
    comment: RwSignal<Comment>,
) -> impl IntoView {
    let state = expect_context::<GlobalState>();
    let show_dialog = RwSignal::new(false);
    let show_button = move || state.user.with(|result| match result {
        Some(Ok(Some(user))) => user.user_id == author_id,
        _ => false,
    });
    let comment_button_class = move || match show_dialog.get() {
        true => "btn btn-circle btn-sm btn-primary",
        false => "btn btn-circle btn-sm btn-ghost",
    };

    view! {
        <Show when=show_button>
            <div>
                <button
                    class=comment_button_class
                    aria-expanded=move || show_dialog.get().to_string()
                    aria-haspopup="dialog"
                    on:click=move |_| show_dialog.update(|show: &mut bool| *show = !*show)
                >
                    <EditIcon/>
                </button>
                <EditCommentDialog
                    comment_id
                    comment
                    show_dialog
                />
            </div>
        </Show>
    }
}

/// Dialog to edit a comment
#[component]
pub fn EditCommentDialog(
    comment_id: i64,
    comment: RwSignal<Comment>,
    show_dialog: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <ModalDialog
            class="w-full max-w-xl"
            show_dialog
        >
            <EditCommentForm
                comment_id
                comment
                show_form=show_dialog
            />
        </ModalDialog>
    }
}

/// Form to edit a comment
#[component]
pub fn EditCommentForm(
    comment_id: i64,
    comment: RwSignal<Comment>,
    show_form: RwSignal<bool>,
) -> impl IntoView {
    let (current_body, is_markdown) =
        comment.with_untracked(|comment| match &comment.markdown_body {
            Some(body) => (body.clone(), true),
            None => (comment.body.clone(), false),
        });
    let comment_body = RwSignal::new(current_body);
    let is_comment_empty =
        move || comment_body.with(|comment_body: &String| comment_body.is_empty());

    let edit_comment_action = ServerAction::<EditComment>::new();

    let edit_comment_result = edit_comment_action.value();
    let has_error = move || edit_comment_result.with(|val| matches!(val, Some(Err(_))));

    Effect::new(move |_| {
        if let Some(Ok(edited_comment)) = edit_comment_result.get() {
            comment.set(edited_comment);
            show_form.set(false);
        }
    });

    view! {
        <div class="bg-base-100 shadow-xl p-3 rounded-sm flex flex-col gap-3">
            <div class="text-center font-bold text-2xl">"Edit your comment"</div>
            <ActionForm action=edit_comment_action>
                <div class="flex flex-col gap-3 w-full">
                    <input
                        type="text"
                        name="comment_id"
                        class="hidden"
                        value=comment_id
                    />
                    <FormMarkdownEditor
                        name="comment"
                        is_markdown_name="is_markdown"
                        placeholder="Your comment..."
                        content=comment_body
                        is_markdown
                    />
                    <ModalFormButtons
                        disable_publish=is_comment_empty
                        show_form
                    />
                </div>
            </ActionForm>
            <ActionError has_error/>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::comment::CommentSortType;
    use crate::constants::{BEST_STR, RECENT_STR};

    #[test]
    fn test_post_sort_type_display() {
        assert_eq!(CommentSortType::Best.to_string(), BEST_STR);
        assert_eq!(CommentSortType::Recent.to_string(), RECENT_STR);
    }
}