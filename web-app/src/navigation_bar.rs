use leptos::*;

static SIDEBAR_TOGGLE_BUTTON_NAME: &str = "Rust";

/// Navigation bar component
#[component]
pub fn NavigationBar<F>(
    cx: Scope,
    on_sidebar_icon_click: F) -> impl IntoView
    where
        F: Fn(leptos::ev::MouseEvent) + 'static, {
    view! { cx,
        <header class="bg-blue-500">
            <div class="container mx-auto p-2 flex flex-wrap flex-row items-center justify-between">
                <div class="flex flex-wrap flex-row gap-4">
                    //<!-- SVG button to open sidebar (hidden on large screens) -->
                    /*<button name=SIDEBAR_TOGGLE_BUTTON_NAME on:click=on_sidebar_icon_click class="lg:hidden">
                      //<!-- Insert your SVG icon here -->
                        <SideBarIcon/>
                    </button>*/
                    <label for="my-drawer" class="drawer-button lg:hidden"><SideBarIcon/></label>
                    //<!-- SVG logo -->
                    <div class="text-white text-lg font-bold">
                      //<!-- Insert your SVG logo here -->
                        <StacksIcon/>
                    </div>
                </div>
                //<!-- Text input -->
                <input type="text" placeholder="Search" class="px-4 py-2 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-600"/>
                <div>
                    //<!-- Login button -->
                    <button class="bg-white text-blue-500 px-4 py-2 rounded-md mx-2">"Login"</button>
                    //<!-- Parameter button -->
                    <button class="bg-white text-blue-500 px-4 py-2 rounded-md">"Parameters"</button>
                </div>
            </div>
        </header>
    }
}

#[component]
pub fn StacksIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg class="w-10 h-10 text-white p-2 bg-indigo-500 rounded-full"
             fill="none"
             stroke="currentColor"
             stroke-linecap="round"
             stroke-linejoin="round"
             stroke-width="2"
             viewBox="0 0 24 24">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
        </svg>
    }
}

#[component]
pub fn RightArrowIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg class="w-4 h-4 ml-1"
             fill="none"
             stroke="currentColor"
             stroke-linecap="round"
             stroke-linejoin="round"
             stroke-width="2"
             viewBox="0 0 24 24">
            <path d="M5 12h14M12 5l7 7-7 7"></path>
        </svg>
    }
}

#[component]
pub fn SideBarIcon(cx: Scope) -> impl IntoView {
    view! { cx,
        <svg class="w-10 h-10 text-white p-2"
             fill="white"
             stroke="currentColor"
             stroke-linecap="round"
             stroke-linejoin="round"
             stroke-width="2"
             viewBox="0 0 448 512">
            <path d="M0 96C0 78.3 14.3 64 32 64H416c17.7 0 32 14.3 32 32s-14.3 32-32 32H32C14.3 128 0 113.7 0 96zM0 256c0-17.7 14.3-32 32-32H416c17.7 0 32 14.3 32 32s-14.3 32-32 32H32c-17.7 0-32-14.3-32-32zM448 416c0 17.7-14.3 32-32 32H32c-17.7 0-32-14.3-32-32s14.3-32 32-32H416c17.7 0 32 14.3 32 32z"/>
        </svg>
    }
}