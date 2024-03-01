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
        <svg class="w-7 h-7 text-white p-1 bg-indigo-500 rounded-full"
             fill="none"
             stroke="currentColor"
             stroke-linecap="round"
             stroke-linejoin="round"
             stroke-width="1.5"
             viewBox="0 0 24 24">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
        </svg>
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
pub fn HotIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 6)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class xmlns="http://www.w3.org/2000/svg" shape-rendering="geometricPrecision" text-rendering="geometricPrecision" image-rendering="optimizeQuality" fill-rule="evenodd" clip-rule="evenodd" viewBox="0 0 384 511.4">
            <defs>
                <linearGradient id="a" gradientUnits="userSpaceOnUse" x1="163.52" y1="286.47" x2="163.52" y2="500.71">
                    <stop offset="0" stop-color="#FB6404"/>
                    <stop offset="1" stop-color="#F2BE10"/>
                </linearGradient>
            </defs>
            <path fill="#E20919" d="M77.46 228.43C65.33 119.85 128.78 43.48 247.72 0c-72.85 94.5 62.09 196.88 69.53 295.03 17.44-29.75 27.34-69.48 29.3-122.55 89.18 139.92 15.25 368.59-181.02 335.73-18.02-3.01-35.38-8.7-51.21-17.17C42.76 452.8 0 369.53 0 290c0-50.69 21.68-95.95 49.74-131.91 3.75 35.23 11.73 61.51 27.72 70.34z"/>
            <path fill="url(#a)" d="M139.16 372.49c-21.83-57.66-18.81-150.75 42.33-183.41.43 107.03 103.57 120.64 84.44 234.9 17.64-20.39 26.51-53.02 28.1-78.75 27.96 65.38 6.04 117.72-33.81 144.37-121.15 81-225.48-83.23-156.11-173.26 2.08 20.07 26.14 51.12 35.05 56.15z"/>
        </svg>
    }
}

#[component]
pub fn SimpleFlameIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 6)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class width="800px" height="800px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <g id="SVGRepo_bgCarrier" stroke-width="0"/>
            <g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"/>
            <g id="SVGRepo_iconCarrier">
                <path
                    d="M5.926 20.574a7.26 7.26 0 0 0 3.039 1.511c.107.035.179-.105.107-.175-2.395-2.285-1.079-4.758-.107-5.873.693-.796 1.68-2.107 1.608-3.865 0-.176.18-.317.322-.211 1.359.703 2.288 2.25 2.538 3.515.394-.386.537-.984.537-1.511 0-.176.214-.317.393-.176 1.287 1.16 3.503 5.097-.072 8.19-.071.071 0 .212.072.177a8.761 8.761 0 0 0 3.003-1.442c5.827-4.5 2.037-12.48-.43-15.116-.321-.317-.893-.106-.893.351-.036.95-.322 2.004-1.072 2.707-.572-2.39-2.478-5.105-5.195-6.441-.357-.176-.786.105-.75.492.07 3.27-2.063 5.352-3.922 8.059-1.645 2.425-2.717 6.89.822 9.808z"
                    fill="#ff9a00"
                />
            </g>
        </svg>
    }
}

#[component]
pub fn FlameIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 6)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class height="256px" width="256px" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 64 64" xml:space="preserve" fill="#000000">
            <defs>
                <linearGradient id="a" gradientUnits="userSpaceOnUse" x1="163.52" y1="286.47" x2="163.52" y2="500.71">
                    <stop offset="0" stop-color="#FB6404"/>
                    <stop offset="1" stop-color="#F2BE10"/>
                </linearGradient>
            </defs>
            <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
            <g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g>
            <g id="SVGRepo_iconCarrier">
                <g id="Layer_1">
                    <g>
                        <circle fill="#C10000" cx="32" cy="32" r="32"></circle>
                    </g>
                    <g opacity="0.5">
                        <path fill="#231F20" d="M28.1,58.1c0,0-16.1-2.4-16.1-16.1s21-15,16.4-32c0,0,15.7,4.9,11.9,20.2c0,0,2.1-1.3,3.7-3.9 c0,0,8,6.2,8,15.5s-11,16.2-16.3,16.2c0,0,5.6-7.6,0.5-12.5c-7.3-7-4.2-11.4-4.2-11.4S14.2,42.8,28.1,58.1z"></path>
                    </g>
                    <g>
                        <path fill="#ff9a00" d="M28.1,56.1c0,0-16.1-2.4-16.1-16.1s21-15,16.4-32c0,0,15.7,4.9,11.9,20.2c0,0,2.1-1.3,3.7-3.9 c0,0,8,6.2,8,15.5s-11,16.2-16.3,16.2c0,0,5.6-7.6,0.5-12.5c-7.3-7-4.2-11.4-4.2-11.4S14.2,40.8,28.1,56.1z"></path>
                    </g>
                </g>
                <g id="Layer_2"></g>
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
pub fn PodiumIcon2(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class fill="white" height="800px" width="800px" version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 512 512" xml:space="preserve">
            <g>
                <g>
                    <g>
                        <path d="M511.915,266.261c-0.021-0.576-0.235-1.131-0.341-1.707c-0.171-0.832-0.299-1.664-0.661-2.411
                        c-0.043-0.085-0.021-0.192-0.064-0.277L489.515,219.2c-1.813-3.584-5.504-5.867-9.536-5.867H362.667v-96
                        c0-0.171-0.085-0.299-0.085-0.448c-0.021-0.427-0.192-0.832-0.256-1.259c-0.171-1.003-0.341-1.963-0.768-2.859
                        c-0.021-0.064-0.021-0.128-0.043-0.192l-21.333-42.667c-1.835-3.627-5.525-5.909-9.557-5.909H181.291
                        c-4.032,0-7.723,2.283-9.536,5.888l-21.333,42.667c-0.064,0.128-0.043,0.277-0.107,0.405c-0.213,0.469-0.256,1.003-0.405,1.493
                        c-0.256,0.875-0.512,1.749-0.533,2.645c0,0.085-0.043,0.149-0.043,0.235V192H32c-4.032,0-7.723,2.283-9.536,5.888L1.131,240.555
                        c-0.043,0.107-0.043,0.213-0.085,0.32c-0.299,0.661-0.405,1.387-0.576,2.112c-0.149,0.661-0.384,1.301-0.405,1.984
                        c0,0.128-0.064,0.235-0.064,0.363v192C0,443.221,4.779,448,10.667,448h490.667c5.888,0,10.667-4.779,10.667-10.667V266.667
                        C512,266.517,511.915,266.389,511.915,266.261z M362.667,234.667h110.72L484.053,256H362.667V234.667z M187.904,85.333h136.149
                        l10.667,21.333H177.237L187.904,85.333z M38.592,213.333h110.741v21.333H27.925L38.592,213.333z M490.667,426.667H21.333V256H160
                        c5.888,0,10.667-4.779,10.667-10.667V128h170.667v95.893c0,0.043-0.021,0.064-0.021,0.107s0.021,0.064,0.021,0.085v42.581
                        c0,5.888,4.779,10.667,10.667,10.667h138.667V426.667z"/>
                        <path d="M270.763,171.456c-3.989-1.664-8.576-0.768-11.627,2.304l-21.333,21.333c-4.16,4.16-4.16,10.923,0,15.083
                        c4.16,4.16,10.923,4.16,15.083,0l3.115-3.115V288c0,5.888,4.779,10.667,10.667,10.667s10.667-4.779,10.688-10.667V181.333
                        C277.355,177.003,274.752,173.12,270.763,171.456z"/>
                        <path d="M426.667,384c-5.888,0-10.667-4.8-10.667-10.667c0-5.888-4.779-10.667-10.667-10.667
                        c-5.888,0-10.667,4.779-10.667,10.667c0,17.643,14.357,32,32,32s32-14.357,32-32c0-8.192-3.093-15.659-8.171-21.333
                        c5.077-5.675,8.171-13.141,8.171-21.333c0-17.643-14.357-32-32-32s-32,14.357-32,32c0,5.888,4.779,10.667,10.667,10.667
                        c5.888,0,10.667-4.779,10.667-10.667C416,324.8,420.779,320,426.667,320s10.667,4.8,10.667,10.667
                        c0,5.867-4.779,10.667-10.667,10.667S416,346.112,416,352c0,5.888,4.779,10.667,10.667,10.667s10.667,4.8,10.667,10.667
                        C437.333,379.179,432.555,384,426.667,384z"/>
                        <path d="M117.333,309.291c0-17.643-14.357-32-32-32c-17.643,0-32,14.357-32,32c0,5.888,4.779,10.667,10.667,10.667
                        s10.667-4.779,10.667-10.667c0-5.867,4.779-10.667,10.667-10.667S96,303.424,96,309.291c0,10.901-4.267,21.163-11.968,28.885
                        L56.448,365.76c-3.051,3.051-3.968,7.637-2.304,11.627C55.787,381.397,59.691,384,64,384h42.667
                        c5.888,0,10.667-4.8,10.667-10.688c0-5.888-4.779-10.667-10.667-10.667H89.749l9.365-9.365
                        C110.869,341.504,117.333,325.888,117.333,309.291z"/>
                    </g>
                </g>
            </g>
        </svg>
    }
}

#[component]
pub fn MedalIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 6)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class width="800px" height="800px" viewBox="0 0 64 64" xmlns="http://www.w3.org/2000/svg">
            <g id="Flat">
                <g id="Color">
                    <polygon fill="#212529" points="45 17 32 25 19 17 19 3 45 3 45 17"/>
                    <polygon fill="#dd051d" points="40 3 40 20.08 32 25 24 20.08 24 3 40 3"/>
                    <path d="M32,25l6.52-4-.17,0a8.22,8.22,0,0,1-1.76-1A8.12,8.12,0,0,0,32,18a8.12,8.12,0,0,0-4.59,1.91,8.22,8.22,0,0,1-1.76,1l-.17,0Z" fill="#a60416"/>
                    <path d="M50.55,40.5c0-2.11,1.57-4.44,1-6.34S48.2,31.24,47,29.6s-1.3-4.48-3-5.69-4.35-.42-6.32-1.05S34.11,20,32,20s-3.83,2.24-5.73,2.86-4.68-.14-6.32,1.05-1.75,4-3,5.69-3.85,2.59-4.49,4.56.95,4.23.95,6.34-1.57,4.44-.95,6.34S15.8,49.76,17,51.4s1.3,4.48,3,5.69,4.35.42,6.32,1S29.89,61,32,61s3.83-2.24,5.73-2.86,4.68.14,6.32-1,1.75-4,3-5.69,3.85-2.59,4.49-4.56S50.55,42.61,50.55,40.5Z" fill="#fccd1d"/>
                    <circle cx="32" cy="40.5" fill="#ffffff" r="14.5"/>
                    <path d="M33.37,33l1.52,2.63a1.54,1.54,0,0,0,1.06.76L39,37a1.53,1.53,0,0,1,.85,2.56l-2.1,2.22a1.5,1.5,0,0,0-.4,1.22l.36,3a1.57,1.57,0,0,1-2.22,1.58l-2.81-1.27a1.6,1.6,0,0,0-1.32,0l-2.81,1.27A1.57,1.57,0,0,1,26.31,46l.36-3a1.5,1.5,0,0,0-.4-1.22l-2.1-2.22A1.53,1.53,0,0,1,25,37l3-.59a1.54,1.54,0,0,0,1.06-.76L30.63,33A1.59,1.59,0,0,1,33.37,33Z" fill="#fccd1d"/>
                </g>
            </g>
        </svg>
    }
}

#[component]
pub fn MedalIcon2(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class height="800px" width="800px" version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 392.533 392.533" xml:space="preserve">
        <path style="fill:#FFC10D;" d="M283.415,283.604c0-48.032-39.111-87.273-87.273-87.273c-48.032,0-87.273,39.111-87.273,87.273
        s39.176,87.273,87.337,87.273C244.369,370.812,283.415,331.636,283.415,283.604z"/>
        <g>
            <polygon style="fill:#56ACE0;" points="209.654,117.527 226.72,141.382 230.599,141.382 316.126,21.851 278.114,21.851 	"/>
            <polygon style="fill:#56ACE0;" points="199.892,141.382 114.365,21.851 76.353,21.851 161.88,141.382 	"/>
        </g>
        <g>
            <path style="fill:#194F82;" d="M337.266,0.065H272.49c-3.556,0-6.788,1.681-8.857,4.59l-67.362,94.19L128.91,4.59
            C126.906,1.745,123.609,0,120.054,0H55.149c-8.469,0-14.222,9.244-8.857,17.261l89.923,125.737
            c-3.168,1.939-5.236,5.301-5.236,9.244v44.154c-26.505,19.911-43.766,51.459-43.766,87.079
            c0,60.121,48.937,109.059,109.059,109.059S305.33,343.596,305.33,283.475c0-35.62-17.261-67.232-43.766-87.079v-44.154
            c0-3.943-2.069-7.37-5.236-9.244l89.859-125.737C351.682,9.438,345.799,0.065,337.266,0.065z M109.064,283.604
            c0-48.032,39.111-87.273,87.273-87.273c48.032,0,87.273,39.111,87.273,87.273s-39.176,87.273-87.273,87.273
            C148.11,370.812,109.064,331.636,109.064,283.604z M239.714,183.661c-13.317-5.883-27.992-9.115-43.442-9.115
            s-30.125,3.232-43.507,9.115v-20.428h87.014v20.428H239.714z M114.365,21.851l85.463,119.531h-38.012L76.353,21.851H114.365z
             M230.599,141.382h-3.943l-17.002-23.855l68.461-95.677h38.012L230.599,141.382z"/>
            <path style="fill:#194F82;" d="M166.664,244.299c-4.719,3.814-5.495,10.667-1.681,15.321c3.814,4.719,10.667,5.495,15.321,1.681
            l6.465-5.172v66.586h-13.317c-6.012,0-10.925,4.848-10.925,10.925s4.848,10.925,10.925,10.925h45.446
            c6.012,0,10.925-4.848,10.925-10.925c0-6.012-4.848-10.925-10.925-10.925h-10.279v-89.277c0-4.202-2.392-8.016-6.206-9.826
            c-3.814-1.745-8.275-1.293-11.507,1.293L166.664,244.299z"/>
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
        <svg class=class width="800px" height="800px" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            <path fill-rule="evenodd" clip-rule="evenodd" fill="#F29C1F" d="M80 100L56 0H44L20 100h13l17-73.914L67 100z"/>
            <path fill-rule="evenodd" clip-rule="evenodd" fill="#ECF0F1" d="M0 10h100v62H0V10z"/>
            <path clip-rule="evenodd" stroke="#E64C3C" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="10" d="M10 61l13.024-13.024L29 53l16.988-16.012L55 46l15-15 5 5 15-15" fill="none"/>
            <path d="M73.28 72H60.56l.46 2h12.74zm-47.04 2h12.74l.46-2H26.72z" fill-rule="evenodd" clip-rule="evenodd" fill="#E57E25"/>
        </svg>
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
        <svg class=class width="800px" height="800px" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
            <path d="M23.36 116.32v-7.42c7.4-1.9 67.86 0 81.28 0v7.42c0 4.24-18.2 7.68-40.64 7.68s-40.64-3.44-40.64-7.68z" fill="#8b5738"/>
            <ellipse cx="64" cy="108.48" rx="40.64" ry="7.68" fill="#ffb17a"/>
            <ellipse cx="64" cy="108.48" rx="40.64" ry="7.68" fill="#cc8552"/>
            <path d="M69.96 65.49c-.75-.31-1.07-.92-1.07-1.73c0-.81.25-1.39.98-1.64c4.61-1.86 27.77-10.73 27.77-38.36l-.18-4.82l-66.98-.08l-.12 5.07c0 26.79 23.08 36.25 27.68 38.11c.75.31 1.22.82 1.22 1.73s-.39 1.39-1.13 1.64c-4.61 1.86-27.77 10.73-27.77 38.36a6.95 6.95 0 0 0 5.34 6.5c5.04 1.19 14.38 2.57 30.53 2.57c13.91 0 21.7-1.01 26.03-2.03c3.08-.73 5.29-3.44 5.36-6.6l.01-.61c.01-26.79-23.06-36.25-27.67-38.11z" opacity=".75" fill="#81d4fa"/>
            <path d="M97.46 18.94l-66.98-.08l-.11 4.52S37.62 27.1 64 27.1s33.63-3.72 33.63-3.72l-.17-4.44z" opacity=".39" fill="#1d44b3"/>
            <path d="M23.36 17.94v-7.87c7.18-.96 70.91 0 81.28 0v7.87c0 3.36-18.2 6.08-40.64 6.08s-40.64-2.72-40.64-6.08z" fill="#8b5738"/>
            <ellipse cx="64" cy="10.08" rx="40.64" ry="6.08" fill="#cc8552"/>
            <g>
                <path d="M90.59 108.57c.92-.27 1.42-1.31.97-2.16c-3.14-5.94-16.54-6.11-21.61-17.27c-3.38-7.45-3.57-17.81-3.67-22.24c-.14-5.99 2.85-7.28 2.85-7.28c14.16-5.7 24.57-18.86 25.17-30.61c.06-1.17-22.18 9.17-29.83 10.66c-14.14 2.76-28.23-.87-28.31-.37c5.24 11.47 15.79 17.46 22.86 20.32c1.68.69 4.46 3.3 4.37 11.14c-.07 5.61-.77 20.4-10.44 26.69c-3.64 2.37-11.69 5.84-13.19 9.61c-.33.83.14 1.77 1.01 1.99c2.76.7 11.18 1.93 24.27 1.93c10.29.01 20.45-.93 25.55-2.41z" fill="#ffca28"/>
                <path d="M42.37 43.29c5.36 2.77 17.12 6.72 22.92 4.72s28.23-16.01 29-19c.96-3.7-26 5.71-35.49 7.91c-6.43 1.49-18.71.72-21.47 1.3c-2.75.57.11 2.52 5.04 5.07z" fill="#e2a610"/>
            </g>
            <g opacity=".6">
                <path d="M45.79 37.66c1.26 2.94 3.56 9.61.56 10.75c-3 1.15-7.39-3.11-9.47-7.39s-1.89-9.96 1.25-10.05c3.14-.09 5.99 2.8 7.66 6.69z" fill="#ffffff"/>
            </g>
            <g opacity=".6">
                <path d="M42.9 80.6c-3.13 3.66-5.48 8.58-4.59 13.33c.94 5.01 5.6 3.63 7.22 2.36c5.16-4.05 3.75-9.24 7.74-15.07c.68-1 3.52-4.13 3.12-6.1c-.24-1.17-2.96-1.77-7.91.71c-2.18 1.1-3.97 2.9-5.58 4.77z" fill="#ffffff"/>
            </g>
        </svg>
    }
}

#[component]
pub fn TimewatchIcon(
    #[prop(optional)]
    class: &'static str,
    #[prop(default = 7)]
    size: i32,
) -> impl IntoView {
    let class = String::from(class) + format!(" h-{size} w-{size}").as_ref();
    view! {
        <svg class=class height="800px" width="800px" version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 512.001 512.001" xml:space="preserve">
            <circle style="fill:#CEE8FA;" cx="256.008" cy="280.395" r="150.734"/>
            <g>
                <path style="fill:#2D527C;" d="M217.384,227.031c16.934,0,34.015,10.657,34.015,30.22c0,31.68-45.111,42.045-45.111,55.33v1.314
                h38.25c3.649,0,6.861,4.526,6.861,9.781s-3.211,10.073-6.861,10.073h-51.973c-4.087,0-9.927-2.773-9.927-7.008v-14.161
                c0-22.92,45.987-36.06,45.987-54.6c0-4.672-2.919-10.365-11.095-10.365c-5.838,0-10.949,2.919-10.949,10.949
                c0,4.235-4.526,8.322-11.972,8.322c-5.84,0-10.219-2.629-10.219-11.826C184.391,237.249,200.742,227.031,217.384,227.031z"/>
                <path style="fill:#2D527C;" d="M293.302,311.266H259.14c-4.233,0-7.591-2.773-7.591-7.883c0-1.168,0.292-2.775,1.168-4.379
                l33.723-66.133c2.189-4.233,5.84-5.84,9.489-5.84c3.941,0,11.388,3.357,11.388,8.613c0,0.876-0.292,1.752-0.73,2.773
                l-25.839,51.827h12.554v-10.219c0-4.818,5.694-6.861,11.388-6.861s11.388,2.044,11.388,6.861v10.219h6.278
                c4.672,0,7.008,5.256,7.008,10.511c0,5.254-3.505,10.51-7.008,10.51h-6.278v15.475c0,4.673-5.694,7.008-11.388,7.008
                s-11.388-2.335-11.388-7.008v-15.474H293.302z"/>
                <path style="fill:#2D527C;" d="M473.008,265.786c-8.065,0-14.603,6.538-14.603,14.603c0,111.606-90.798,202.406-202.406,202.406
                S53.595,391.996,53.595,280.389S144.393,77.985,256.001,77.985c44.436,0,87.346,14.551,122.436,41.233l-26.546,26.546
                c-27.063-19.332-60.171-30.719-95.89-30.719c-91.17,0-165.344,74.172-165.344,165.344s74.172,165.344,165.344,165.344
                s165.344-74.172,165.344-165.344c0-37.329-12.44-71.804-33.386-99.509l80.046,21.448c1.25,0.334,2.519,0.498,3.779,0.498
                c0.055,0,0.111,0,0.166,0c8.001-0.073,14.466-6.583,14.466-14.603c0-1.647-0.273-3.232-0.775-4.708L455.946,72.689
                c-1.351-5.04-5.286-8.975-10.326-10.326c-5.042-1.351-10.418,0.091-14.107,3.779l-32.26,32.26
                c-36.911-29.081-81.727-46.203-128.649-49.154V29.206h17.717c8.065,0,14.603-6.538,14.603-14.603S296.386,0,288.32,0h-64.639
                c-8.065,0-14.603,6.538-14.603,14.603s6.538,14.603,14.603,14.603h17.717v20.046C120.466,56.815,24.389,157.584,24.389,280.389
                c0,127.71,103.9,231.612,231.612,231.612s231.612-103.9,231.612-231.612C487.611,272.324,481.073,265.786,473.008,265.786z
                 M256.001,416.527c-75.066,0-136.138-61.072-136.138-136.138s61.07-136.138,136.138-136.138s136.138,61.07,136.138,136.138
                S331.067,416.527,256.001,416.527z M434.28,104.68l16.852,62.891l-62.891-16.852L434.28,104.68z"/>
            </g>
        </svg>
    }
}