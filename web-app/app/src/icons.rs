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
                    <stop offset="0" stop-color="#ffac2f"/>
                    <stop offset="1" stop-color="#fff073"/>
                </linearGradient>
            </defs>
            <path fill="#f77d02" d="M77.46 228.43C65.33 119.85 128.78 43.48 247.72 0c-72.85 94.5 62.09 196.88 69.53 295.03 17.44-29.75 27.34-69.48 29.3-122.55 89.18 139.92 15.25 368.59-181.02 335.73-18.02-3.01-35.38-8.7-51.21-17.17C42.76 452.8 0 369.53 0 290c0-50.69 21.68-95.95 49.74-131.91 3.75 35.23 11.73 61.51 27.72 70.34z"/>
            <path fill="url(#a)" d="M139.16 372.49c-21.83-57.66-18.81-150.75 42.33-183.41.43 107.03 103.57 120.64 84.44 234.9 17.64-20.39 26.51-53.02 28.1-78.75 27.96 65.38 6.04 117.72-33.81 144.37-121.15 81-225.48-83.23-156.11-173.26 2.08 20.07 26.14 51.12 35.05 56.15z"/>
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
        <svg class=class width="800px" height="800px" viewBox="-33 0 255 255" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" preserveAspectRatio="xMidYMid">
            <defs>
                <linearGradient id="linear-gradient-1" gradientUnits="userSpaceOnUse" x1="94.141" y1="255" x2="94.141" y2="0.188">
                    <stop offset="0" stop-color="#f77d02"/>
                    <stop offset="1" stop-color="#f77d02"/>
                </linearGradient>
            </defs>
            <g id="fire">
                <path
                    id="path-1" fill="url(#linear-gradient-1)" fill-rule="evenodd"
                    d="M187.899,164.809 C185.803,214.868 144.574,254.812 94.000,254.812 C42.085,254.812 -0.000,211.312 -0.000,160.812 C-0.000,154.062 -0.121,140.572 10.000,117.812 C16.057,104.191 19.856,95.634 22.000,87.812 C23.178,83.513 25.469,76.683 32.000,87.812 C35.851,94.374 36.000,103.812 36.000,103.812 C36.000,103.812 50.328,92.817 60.000,71.812 C74.179,41.019 62.866,22.612 59.000,9.812 C57.662,5.384 56.822,-2.574 66.000,0.812 C75.352,4.263 100.076,21.570 113.000,39.812 C131.445,65.847 138.000,90.812 138.000,90.812 C138.000,90.812 143.906,83.482 146.000,75.812 C148.365,67.151 148.400,58.573 155.999,67.813 C163.226,76.600 173.959,93.113 180.000,108.812 C190.969,137.321 187.899,164.809 187.899,164.809 Z"/>
                <path
                    id="path-2" fill="#ffbd42" fill-rule="evenodd"
                    d="M94.000,254.812 C58.101,254.812 29.000,225.711 29.000,189.812 C29.000,168.151 37.729,155.000 55.896,137.166 C67.528,125.747 78.415,111.722 83.042,102.172 C83.953,100.292 86.026,90.495 94.019,101.966 C98.212,107.982 104.785,118.681 109.000,127.812 C116.266,143.555 118.000,158.812 118.000,158.812 C118.000,158.812 125.121,154.616 130.000,143.812 C131.573,140.330 134.753,127.148 143.643,140.328 C150.166,150.000 159.127,167.390 159.000,189.812 C159.000,225.711 129.898,254.812 94.000,254.812 Z"/>
                <path
                    id="path-3" fill="#fff073" fill-rule="evenodd"
                    d="M95.000,183.812 C104.250,183.812 104.250,200.941 116.000,223.812 C123.824,239.041 112.121,254.812 95.000,254.812 C77.879,254.812 69.000,240.933 69.000,223.812 C69.000,206.692 85.750,183.812 95.000,183.812 Z"/>
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
pub fn MedalIcon(
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