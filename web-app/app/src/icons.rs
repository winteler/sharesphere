use leptos::*;

/// Renders a loading icon
#[component]
pub fn LoadingIcon() -> impl IntoView {
    view! {
        <span class="loading loading-spinner loading-md"></span>
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
pub fn UserIcon(
    #[prop(default = "h-7 w-7")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/user.svg" class=class/>
    }
}

#[component]
pub fn LogoIcon(
    #[prop(default = "h-7 w-7")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/planet.svg" class=class/>
    }
}

#[component]
pub fn PlusIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView  {
    view! {
        <img src="/svg/plus.svg" class=class/>
    }
}

#[component]
pub fn MinusIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/minus.svg" class=class/>
    }
}

#[component]
pub fn SideBarIcon(
    #[prop(default = "h-6 w-6")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/sidebar.svg" class=class/>
    }
}

#[component]
pub fn SearchIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/search.svg" class=class/>
    }
}

#[component]
pub fn ScoreIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/score.svg" class=class/>
    }
}

#[component]
pub fn CommentIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/comment.svg" class=class/>
    }
}

#[component]
pub fn AuthorIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/author.svg" class=class/>
    }
}
#[component]
pub fn ClockIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/clock.svg" class=class/>
    }
}

#[component]
pub fn BoldIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/bold_font.svg" class=class/>
    }
}

#[component]
pub fn MinimizeIcon(
    #[prop(default = "h-6 w-6")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/minimize.svg"/>
        </div>
    }
}

#[component]
pub fn MaximizeIcon(
    #[prop(default = "h-6 w-6")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <div class=class>
            <img src="/svg/maximize.svg"/>
        </div>
    }
}

#[component]
pub fn FlameIcon(
    #[prop(default = "h-7 w-7")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/flame.svg" class=class/>
    }
}

#[component]
pub fn PodiumIcon(
    #[prop(default = "h-7 w-7")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/podium.svg" class=class/>
    }
}

#[component]
pub fn GraphIcon(
    #[prop(default = "h-7 w-7")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/graph.svg" class=class/>
    }
}

#[component]
pub fn HourglassIcon(
    #[prop(default = "h-7 w-7")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <img src="/svg/hourglass.svg" class=class/>
    }
}

#[component]
pub fn StarIcon(
    #[prop(default = "h-7 w-7")]
    class: &'static str,
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
    #[prop(default = "h-7 w-7")]
    class: &'static str,
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