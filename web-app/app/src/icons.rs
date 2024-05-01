use leptos::*;

#[component]
pub fn AuthorIcon(#[prop(default = "h-5 w-5")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/author.svg" class=class/>
    }
}

#[component]
pub fn BoldIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/bold.svg" class=class/>
    }
}

#[component]
pub fn ClockIcon(#[prop(default = "h-5 w-5")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/clock.svg" class=class/>
    }
}

#[component]
pub fn CommentIcon(#[prop(default = "h-5 w-5")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/comment.svg" class=class/>
    }
}

#[component]
pub fn ErrorIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-5 w-5" fill="none" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
    }
}

#[component]
pub fn FlameIcon(#[prop(default = "h-7 w-7")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/flame.svg" class=class/>
    }
}

#[component]
pub fn GraphIcon(#[prop(default = "h-7 w-7")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/graph.svg" class=class/>
    }
}

#[component]
pub fn HourglassIcon(#[prop(default = "h-7 w-7")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/hourglass.svg" class=class/>
    }
}

#[component]
pub fn ItalicIcon(#[prop(default = "h-4 w-4")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/editor/italic.svg" class=class/>
    }
}

/// Renders a loading icon
#[component]
pub fn LoadingIcon(#[prop(default = "loading-md my-5")] class: &'static str) -> impl IntoView {
    let class = String::from("loading loading-spinner ") + class;
    view! {
        <div class="w-full flex content-center justify-center">
            <div class=class></div>
        </div>
    }
}

#[component]
pub fn LogoIcon(#[prop(default = "h-7 w-7")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/planet.svg" class=class/>
    }
}

#[component]
pub fn MarkdownIcon(#[prop(default = "h-5 w-10")] class: &'static str) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/editor/markdown.svg"/>
        </div>
    }
}

#[component]
pub fn MinimizeIcon(#[prop(default = "h-6 w-6")] class: &'static str) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/minimize.svg"/>
        </div>
    }
}

#[component]
pub fn MinusIcon(#[prop(default = "h-5 w-5")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/minus.svg" class=class/>
    }
}

#[component]
pub fn MaximizeIcon(#[prop(default = "h-6 w-6")] class: &'static str) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/maximize.svg"/>
        </div>
    }
}

#[component]
pub fn PlusIcon(#[prop(default = "h-5 w-5")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/plus.svg" class=class/>
    }
}

#[component]
pub fn PodiumIcon(#[prop(default = "h-7 w-7")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/podium.svg" class=class/>
    }
}

#[component]
pub fn ScoreIcon(#[prop(default = "h-5 w-5")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/score.svg" class=class/>
    }
}

#[component]
pub fn SearchIcon(#[prop(default = "h-5 w-5")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/search.svg" class=class/>
    }
}

#[component]
pub fn SideBarIcon(#[prop(default = "h-6 w-6")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/sidebar.svg" class=class/>
    }
}

#[component]
pub fn StarIcon(
    #[prop(default = "h-7 w-7")] class: &'static str,
    show_colour: RwSignal<bool>,
) -> impl IntoView {
    let svg_path = move || match show_colour() {
        true => "/svg/stars.svg",
        false => "/svg/stars_disabled.svg",
    };
    view! {
        <img src=svg_path class=class/>
    }
}

#[component]
pub fn SubscribedIcon(
    #[prop(default = "h-7 w-7")] class: &'static str,
    show_colour: RwSignal<bool>,
) -> impl IntoView {
    let svg_path = move || match show_colour() {
        true => "/svg/planet.svg",
        false => "/svg/planet_disabled.svg",
    };
    view! {
        <img src=svg_path class=class/>
    }
}

#[component]
pub fn UserIcon(#[prop(default = "h-7 w-7")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/user.svg" class=class/>
    }
}

#[component]
pub fn NotFoundIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/man_on_the_moon.svg" class=class/>
    }
}

#[component]
pub fn InternalErrorIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/landing_space_capsule.svg" class=class/>
    }
}

#[component]
pub fn AuthErrorIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/alien.svg" class=class/>
    }
}

#[component]
pub fn InvalidRequestIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/chewbacca.svg" class=class/>
    }
}

#[component]
pub fn NetworkErrorIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/satellite.svg" class=class/>
    }
}

#[component]
pub fn NotAuthenticatedIcon(#[prop(default = "h-28 w-28")] class: &'static str) -> impl IntoView {
    view! {
        <img src="/svg/errors/stormtrooper.svg" class=class/>
    }
}

