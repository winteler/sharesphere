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
pub fn UserIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 icon icon-tabler icon-tabler-user stroke-white" width="44" height="44" viewBox="0 0 24 24" fill="none" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
              <path d="M8 7a4 4 0 1 0 8 0a4 4 0 0 0 -8 0" />
              <path d="M6 21v-2a4 4 0 0 1 4 -4h4a4 4 0 0 1 4 4v2" />
        </svg>
    }
}

#[component]
pub fn LogoIcon() -> impl IntoView {
    view! {
        <img src="/svg/planet.svg" class="w-7 h-7 text-white rounded-full"/>
    }
}

#[component]
pub fn PlusIcon(
    #[prop(default = "h-5 w-5")]
    class: &'static str,
) -> impl IntoView  {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class=class width="44" height="44" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M12 5l0 14" />
            <path d="M5 12l14 0" />
        </svg>
    }
}

#[component]
pub fn MinusIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" width="44" height="44" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M5 12l14 0" />
        </svg>
    }
}

#[component]
pub fn SideBarIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-6 h-6 stroke-white">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 6h16M4 12h16M4 18h16"></path>
        </svg>
    }
}

#[component]
pub fn SearchIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="h-5 w-5 stroke-white">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
    }
}

#[component]
pub fn ScoreIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 icon icon-tabler icon-tabler-switch-vertical" width="44" height="44" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M3 8l4 -4l4 4" />
            <path d="M7 4l0 9" />
            <path d="M13 16l4 4l4 -4" />
            <path d="M17 10l0 10" />
        </svg>
    }
}

#[component]
pub fn CommentIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 icon icon-tabler icon-tabler-message-plus" width="44" height="44" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M8 9h8" />
            <path d="M8 13h6" />
            <path d="M12.01 18.594l-4.01 2.406v-3h-2a3 3 0 0 1 -3 -3v-8a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v5.5" />
            <path d="M16 19h6" />
            <path d="M19 16v6" />
        </svg>
    }
}

#[component]
pub fn AuthorIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 icon icon-tabler icon-tabler-user-edit" width="44" height="44" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M8 7a4 4 0 1 0 8 0a4 4 0 0 0 -8 0" />
            <path d="M6 21v-2a4 4 0 0 1 4 -4h3.5" />
            <path d="M18.42 15.61a2.1 2.1 0 0 1 2.97 2.97l-3.39 3.42h-3v-3l3.42 -3.39z" />
        </svg>
    }
}
#[component]
pub fn ClockIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 icon icon-tabler icon-tabler-clock-hour-4" width="44" height="44" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0" />
            <path d="M12 12l3 2" />
            <path d="M12 7v5" />
        </svg>
    }
}

#[component]
pub fn BoldIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 icon icon-tabler icon-tabler-bold" width="44" height="44" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M7 5h6a3.5 3.5 0 0 1 0 7h-6z" />
            <path d="M13 12h1a3.5 3.5 0 0 1 0 7h-7v-7" />
        </svg>
    }
}

#[component]
pub fn MinimizeIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 6)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size} icon icon-tabler icon-tabler-maximize").as_ref();
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class=class width="44" height="44" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M15 19v-2a2 2 0 0 1 2 -2h2" />
            <path d="M15 5v2a2 2 0 0 0 2 2h2" />
            <path d="M5 15h2a2 2 0 0 1 2 2v2" />
            <path d="M5 9h2a2 2 0 0 0 2 -2v-2" />
        </svg>
    }
}

#[component]
pub fn MaximizeIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 5)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size} icon icon-tabler icon-tabler-maximize").as_ref();
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class=class width="44" height="44" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M4 8v-2a2 2 0 0 1 2 -2h2" />
            <path d="M4 16v2a2 2 0 0 0 2 2h2" />
            <path d="M16 4h2a2 2 0 0 1 2 2v2" />
            <path d="M16 20h2a2 2 0 0 0 2 -2v-2" />
        </svg>
    }
}

#[component]
pub fn FlameIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class width="800px" height="800px" viewBox="-33 0 255 255" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" preserveAspectRatio="xMidYMid">
            <defs>
                <linearGradient id="linear-gradient" gradientUnits="userSpaceOnUse" x1="94.141" y1="255" x2="94.141" y2="180">
                    <stop offset="0" stop-color="#ffffc0"/>
                    <stop offset="1" stop-color="#ffbd42"/>
                </linearGradient>
            </defs>
            <g>
                <path
                    fill="#f77d02" fill-rule="evenodd"
                    d="M187.899,164.809 C185.803,214.868 144.574,254.812 94.000,254.812 C42.085,254.812 -0.000,211.312 -0.000,160.812 C-0.000,154.062 -0.121,140.572 10.000,117.812 C16.057,104.191 19.856,95.634 22.000,87.812 C23.178,83.513 25.469,76.683 32.000,87.812 C35.851,94.374 36.000,103.812 36.000,103.812 C36.000,103.812 50.328,92.817 60.000,71.812 C74.179,41.019 62.866,22.612 59.000,9.812 C57.662,5.384 56.822,-2.574 66.000,0.812 C75.352,4.263 100.076,21.570 113.000,39.812 C131.445,65.847 138.000,90.812 138.000,90.812 C138.000,90.812 143.906,83.482 146.000,75.812 C148.365,67.151 148.400,58.573 155.999,67.813 C163.226,76.600 173.959,93.113 180.000,108.812 C190.969,137.321 187.899,164.809 187.899,164.809 Z"/>
                <path
                    fill="url(#linear-gradient)" fill-rule="evenodd"
                    d="M94.000,254.812 C58.101,254.812 29.000,225.711 29.000,189.812 C29.000,168.151 37.729,155.000 55.896,137.166 C67.528,125.747 78.415,111.722 83.042,102.172 C83.953,100.292 86.026,90.495 94.019,101.966 C98.212,107.982 104.785,118.681 109.000,127.812 C116.266,143.555 118.000,158.812 118.000,158.812 C118.000,158.812 125.121,154.616 130.000,143.812 C131.573,140.330 134.753,127.148 143.643,140.328 C150.166,150.000 159.127,167.390 159.000,189.812 C159.000,225.711 129.898,254.812 94.000,254.812 Z"/>
            </g>
        </svg>
    }
}

#[component]
pub fn PodiumIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 491.546 491.546" xml:space="preserve" width="800px" height="800px" fill="#000000">
            <g id="SVGRepo_bgCarrier" stroke-width="0"/>
            <g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"/>
            <g id="SVGRepo_iconCarrier">
                <rect x="0.013" y="241.459" style="fill:#bc3c29;" width="163.84" height="250.035"/>
                <rect x="163.853" y="17.874" style="fill:#31978C;" width="163.84" height="473.651"/>
                <rect x="327.693" y="166.272" style="fill:#DC8744;" width="163.84" height="325.274"/>
                <rect x="0.013" y="223.611" style="fill:#e24f3f;" width="163.84" height="250.035"/>
                <rect x="163.853" style="fill:#44C4A1;" width="163.84" height="473.651"/>
                <rect x="327.693" y="148.398" style="fill:#FCD462;" width="163.84" height="325.274"/>
                <g>
                    <path style="fill:#000000;" d="M224.454,184.161h42.637v105.324h-22.6v-85.287h-20.037V184.161z"/>
                    <path style="fill:#000000;" d="M417.596,300.623c2.661-3.766,3.997-7.407,3.997-10.926c0-3.507-1.152-6.45-3.471-8.817 c-2.306-2.355-5.272-3.532-8.89-3.532c-6.622,0-12.9,4.721-18.836,14.163l-18.835-11.159c4.93-7.628,10.325-13.403,16.199-17.328 c5.874-3.911,13.366-5.874,22.453-5.874c9.086,0,17.082,2.895,23.961,8.67c6.88,5.775,10.326,13.636,10.326,23.581 c0,5.42-1.386,10.62-4.145,15.598c-2.773,4.967-7.861,11.32-15.304,19.057L406.069,343.8h41.595v21.092h-74.287v-17.474 l30.89-31.65C410.495,309.439,414.934,304.387,417.596,300.623z"/>
                    <path style="fill:#000000;" d="M50.148,315.402v-20.037h64.489v16.273L93.091,336.2c8.437,1.41,14.961,4.942,19.584,10.619 c4.623,5.678,6.929,12.227,6.929,19.67c0,11.048-3.74,19.706-11.221,25.985c-7.48,6.279-17.057,9.419-28.706,9.419 c-11.65,0-23.459-4.121-35.415-12.349l9.65-18.689c9.946,7.026,18.836,10.547,26.672,10.547c4.721,0,8.608-1.154,11.674-3.459 c3.067-2.317,4.599-5.653,4.599-10.03c0-4.366-1.766-7.836-5.272-10.399c-3.521-2.551-8.388-3.838-14.618-3.838 c-3.311,0-7.983,0.956-14.016,2.869v-17.34l20.197-23.802H50.148z"/>
                </g>
            </g>
        </svg>
    }
}

#[component]
pub fn GraphIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <img src="/svg/graph.svg" class=class/>
    }
}

#[component]
pub fn HourglassIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <img src="/svg/hourglass.svg" class=class/>
    }
}

#[component]
pub fn StarIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
    show_colour: RwSignal<bool>,
) -> impl IntoView {
    let fill_colour = move || match show_colour() {
        true => "#ffc617",
        false => "#aeaeae",
    };
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class width="256px" height="256px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <g id="SVGRepo_bgCarrier" stroke-width="0"/>
            <g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"/>
            <g id="SVGRepo_iconCarrier">
                <path fill=fill_colour d="M10.2768 16.5148C10.2815 16.405 10.4634 16.3613 10.5174 16.4571C10.7707 16.9068 11.2029 17.5682 11.6932 17.8689C12.1836 18.1696 12.969 18.2549 13.4847 18.2768C13.5945 18.2815 13.6381 18.4634 13.5423 18.5174C13.0926 18.7707 12.4313 19.2029 12.1306 19.6932C11.8299 20.1836 11.7446 20.969 11.7227 21.4847C11.718 21.5945 11.536 21.6381 11.4821 21.5423C11.2287 21.0926 10.7966 20.4313 10.3062 20.1306C9.81588 19.8299 9.03048 19.7446 8.51481 19.7227C8.40495 19.718 8.36133 19.536 8.45713 19.4821C8.90682 19.2287 9.56818 18.7966 9.86889 18.3062C10.1696 17.8159 10.2549 17.0305 10.2768 16.5148Z"/>
                <path fill=fill_colour opacity="0.5" d="M18.4919 15.5147C18.4834 15.4051 18.2916 15.3591 18.2343 15.453C18.062 15.7355 17.8135 16.0764 17.5374 16.2458C17.2612 16.4152 16.8446 16.482 16.5147 16.5075C16.4051 16.516 16.3591 16.7078 16.453 16.7651C16.7355 16.9374 17.0764 17.1858 17.2458 17.462C17.4152 17.7382 17.482 18.1548 17.5075 18.4847C17.516 18.5943 17.7078 18.6403 17.7651 18.5464C17.9374 18.2639 18.1858 17.923 18.462 17.7536C18.7382 17.5842 19.1548 17.5174 19.4847 17.4919C19.5943 17.4834 19.6403 17.2916 19.5464 17.2343C19.2639 17.062 18.923 16.8135 18.7536 16.5374C18.5842 16.2612 18.5174 15.8446 18.4919 15.5147Z"/>
                <path fill=fill_colour d="M14.7034 4.00181L14.4611 3.69574C13.5245 2.51266 13.0561 1.92112 12.5113 2.00845C11.9665 2.09577 11.7059 2.80412 11.1849 4.22083L11.0501 4.58735C10.902 4.98993 10.828 5.19122 10.686 5.33897C10.544 5.48671 10.3501 5.56417 9.96242 5.71911L9.60942 5.86016L9.36156 5.95933C8.16204 6.4406 7.55761 6.71331 7.48044 7.24324C7.39813 7.80849 7.97023 8.29205 9.11443 9.25915L9.41045 9.50935C9.7356 9.78417 9.89817 9.92158 9.99137 10.1089C10.0846 10.2962 10.0978 10.5121 10.1244 10.9441L10.1485 11.3373C10.2419 12.8574 10.2886 13.6174 10.7826 13.8794C11.2765 14.1414 11.8906 13.7319 13.1188 12.9129L13.1188 12.9129L13.4366 12.701C13.7856 12.4683 13.9601 12.3519 14.1597 12.32C14.3593 12.288 14.5613 12.344 14.9655 12.456L15.3334 12.558C16.7555 12.9522 17.4666 13.1493 17.8542 12.746C18.2418 12.3427 18.0493 11.6061 17.6641 10.1328L17.5645 9.75163C17.4551 9.33297 17.4003 9.12364 17.4305 8.91657C17.4606 8.70951 17.5723 8.52816 17.7955 8.16546L17.7955 8.16544L17.9987 7.83522C18.7843 6.55883 19.1771 5.92063 18.9227 5.40935C18.6682 4.89806 17.9351 4.85229 16.4689 4.76076L16.0896 4.73708C15.6729 4.71107 15.4646 4.69807 15.2836 4.60208C15.1027 4.5061 14.9696 4.338 14.7034 4.00181L14.7034 4.00181Z"/>
                <path fill=fill_colour opacity="0.5" d="M8.835 13.326C6.69772 14.3702 4.91931 16.024 4.24844 18.0002C3.49589 13.2926 4.53976 10.2526 6.21308 8.36328C6.35728 8.658 6.54466 8.902 6.71297 9.09269C7.06286 9.48911 7.56518 9.91347 8.07523 10.3444L8.44225 10.6545C8.51184 10.7134 8.56597 10.7592 8.61197 10.7989C8.61665 10.8632 8.62129 10.9383 8.62727 11.0357L8.65708 11.5212C8.69717 12.1761 8.7363 12.8155 8.835 13.326Z"/>
            </g>
        </svg>
    }
}

#[component]
pub fn SubscribedIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
    show_colour: RwSignal<bool>,
) -> impl IntoView {
    let svg_path = move || match show_colour() {
        true => "/svg/planet.svg",
        false => "/svg/planet-disabled.svg",
    };
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <img src=svg_path class=class/>
    }
}