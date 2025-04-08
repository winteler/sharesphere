use leptos::prelude::*;

#[component]
pub fn AddCommentIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/add_comment.svg" class=class/>
    }
}

#[component]
pub fn ArrowUpIcon(
    #[prop(into)]
    class: Signal<String>,
) -> impl IntoView {
    view! {
        <img src="/svg/arrow_up.svg" class=class/>
    }
}

#[component]
pub fn AuthErrorIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/alien.svg" class=class/>
    }
}

#[component]
pub fn AuthorIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/author.svg" class=class/>
    }
}

#[component]
pub fn SelfAuthorIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/author_filled.svg" class=class/>
    }
}

#[component]
pub fn BannedIcon(#[prop(default = "h-20 w-20")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/banned.svg" class=class/>
    }
}

#[component]
pub fn BoldIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/bold.svg" class=class/>
    }
}

#[component]
pub fn ClockIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/clock.svg" class=class/>
    }
}

#[component]
pub fn CodeBlockIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/codeblock.svg" class=class/>
    }
}

#[component]
pub fn CommentIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/comment.svg" class=class/>
    }
}

#[component]
pub fn CrossIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/cross.svg" class=class/>
    }
}

#[component]
pub fn DeleteIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/delete.svg" class=class/>
    }
}

#[component]
pub fn DotMenuIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/dot_menu.svg" class=class/>
    }
}

#[component]
pub fn EditIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/edit.svg" class=class/>
    }
}

#[component]
pub fn EditTimeIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/edit_time.svg" class=class/>
    }
}

#[component]
pub fn SphereIcon(
    icon_url: Option<String>,
    #[prop(default = "h-7 w-7")]
    class: &'static str
) -> impl IntoView {
    match icon_url {
        Some(icon_url) => {
            let class = format!("rounded-full overflow-hidden {class}");
            view! { <img src=icon_url class=class/> }.into_any()
        },
        None => view! { <LogoIcon class/> }.into_any(),
    }
}

#[component]
pub fn FiltersIcon(#[prop(default = "filter-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/filters.svg" class=class/>
    }
}

#[component]
pub fn FlameIcon(#[prop(default = "filter-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/flame.svg" class=class/>
    }
}

#[component]
pub fn GraphIcon(#[prop(default = "filter-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/graph.svg" class=class/>
    }
}

#[component]
pub fn HammerIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/hammer.svg" class=class/>
    }
}

#[component]
pub fn Header1Icon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/header_1.svg" class=class/>
    }
}

#[component]
pub fn Header2Icon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/header_2.svg" class=class/>
    }
}

#[component]
pub fn HelpIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/help.svg" class=class/>
    }
}

#[component]
pub fn HourglassIcon(#[prop(default = "filter-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/hourglass.svg" class=class/>
    }
}

#[component]
pub fn ImageIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/image.svg" class=class/>
    }
}

#[component]
pub fn InternalErrorIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/landing_space_capsule.svg" class=class/>
    }
}

#[component]
pub fn InvalidRequestIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/chewbacca.svg" class=class/>
    }
}

#[component]
pub fn ItalicIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/italic.svg" class=class/>
    }
}

#[component]
pub fn LinkIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/link.svg" class=class/>
    }
}

#[component]
pub fn ListBulletIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/bullet_list.svg" class=class/>
    }
}

#[component]
pub fn ListNumberIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/number_list.svg" class=class/>
    }
}

/// Renders a loading icon
#[component]
pub fn LoadingIcon(#[prop(default = "h-7 w-7 my-5")] class: &'static str) -> impl IntoView {
    view! {
        <div class="w-full flex items-center justify-center">
            <img src="/svg/loading.svg" class=class/>
        </div>
    }
}

#[component]
pub fn LogoIcon(#[prop(default = "navbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/planet.svg" class=class/>
    }
}

#[component]
pub fn MagnifierIcon(#[prop(default = "navbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/magnifier.svg" class=class/>
    }
}

#[component]
pub fn MarkdownIcon(#[prop(default = "h-4 w-8")] class: &'static str) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/editor/markdown.svg"/>
        </div>
    }
}

#[component]
pub fn MaximizeIcon(#[prop(default = "h-4 w-4 2xl:h-6 2xl:w-6")] class: &'static str) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/maximize.svg"/>
        </div>
    }
}

#[component]
pub fn MinimizeIcon(#[prop(default = "h-4 w-4 2xl:h-6 2xl:w-6")] class: &'static str) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/minimize.svg"/>
        </div>
    }
}

#[component]
pub fn MinusIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/minus.svg" class=class/>
    }
}

#[component]
pub fn ModeratorIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/moderator.svg" class=class/>
    }
}

#[component]
pub fn ModeratorAuthorIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/moderator_filled.svg" class=class/>
    }
}

#[component]
pub fn NetworkErrorIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/satellite.svg" class=class/>
    }
}

#[component]
pub fn NotAuthorizedIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/stormtrooper.svg" class=class/>
    }
}

#[component]
pub fn NotFoundIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/man_on_the_moon.svg" class=class/>
    }
}

#[component]
pub fn NsfwIcon() -> impl IntoView {
    view! {
        <div class="rounded-full px-1 pt-1 pb-1.5 bg-black text-sm font-semibold leading-none w-fit h-fit">"18+"</div>
    }
}

#[component]
pub fn PauseIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/pause.svg" class=class/>
    }
}

#[component]
pub fn PinnedIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/pin.svg" class=class/>
    }
}

#[component]
pub fn PlayIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/play.svg" class=class/>
    }
}

#[component]
pub fn PlusIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/plus.svg" class=class/>
    }
}

#[component]
pub fn PodiumIcon(#[prop(default = "filter-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/podium.svg" class=class/>
    }
}

#[component]
pub fn QuoteIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/quote.svg" class=class/>
    }
}

#[component]
pub fn SaveIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/form/save.svg" class=class/>
    }
}

#[component]
pub fn ScoreIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/toolbar/score.svg" class=class/>
    }
}

#[component]
pub fn SettingsIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/settings_gear.svg" class=class/>
    }
}

#[component]
pub fn SideBarIcon(#[prop(default = "navbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/sidebar.svg" class=class/>
    }
}

#[component]
pub fn SpoilerIcon(#[prop(default = "content-toolbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/spoiler.svg" class=class/>
    }
}

#[component]
pub fn StarIcon(
    #[prop(default = "h-7 w-7")] class: &'static str,
    show_color: RwSignal<bool>,
) -> impl IntoView {
    let svg_path = move || match show_color.get() {
        true => "/svg/stars.svg",
        false => "/svg/stars_disabled.svg",
    };
    view! {
        <img src=svg_path class=class/>
    }
}

#[component]
pub fn StrikethroughIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/strikethrough.svg" class=class/>
    }
}

#[component]
pub fn SubscribedIcon(
    #[prop(default = "h-7 w-7")] class: &'static str,
    show_color: RwSignal<bool>,
) -> impl IntoView {
    let svg_path = move || match show_color.get() {
        true => "/svg/planet.svg",
        false => "/svg/planet_disabled.svg",
    };
    view! {
        <img src=svg_path class=class/>
    }
}

#[component]
pub fn TooHeavyIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/weight.svg" class=class/>
    }
}

#[component]
pub fn UserIcon(#[prop(default = "navbar-icon-size")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/user.svg" class=class/>
    }
}

#[component]
pub fn UserSettingsIcon(#[prop(default = "h-7 w-7")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/user_settings.svg" class=class/>
    }
}
