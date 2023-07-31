use leptos::*;

/// Navigation bar component
#[component]
pub fn NavigationBar<F>(cx: Scope, on_sidebar_icon_click: F) -> impl IntoView {


    view! { cx,
        /*<header class="text-gray-400 bg-gray-900 body-font">
            <div class="container mx-auto flex flex-wrap p-5 flex-col md:flex-row items-center">
                <a class="flex title-font font-medium items-center text-white mb-4 md:mb-0">
                    <StacksIcon/>
                </a>
                <nav class="md:ml-auto flex flex-wrap items-center text-base justify-center">
                    <a href="#" class="mr-5 hover:text-white">"Login"</a>
                    <a href="#" class="mr-5 hover:text-white">"Settings"</a>
                </nav>
                <div class="form-control">
                  //<label class="label">
                    //<span class="label-text">"Search"</span>
                    //<span class="label-text-alt">Top Right label</span>
                  //</label>
                  <input type="text" placeholder="Search" class="input input-bordered w-full max-w-xs" />
                  //<label class="label">
                    //<span class="label-text-alt">Bottom Left label</span>
                    //<span class="label-text-alt">Bottom Right label</span>
                  //</label>
                </div>
                <button class="inline-flex items-center bg-gray-800 border-0 py-1 px-3 focus:outline-none hover:bg-gray-700 rounded text-base mt-4 md:mt-0">
                    "Button"
                    <RightArrowIcon/>
                </button>
            </div>
        </header>
             */
        <header class="bg-blue-500">
            <div class="container mx-auto p-4 flex flex-wrap flex-row items-center justify-between">
                <div class="flex flex-wrap flex-row gap-4">
                    //<!-- SVG button to open sidebar (hidden on large screens) -->
                    <button class="lg:hidden">
                      //<!-- Insert your SVG icon here -->
                        <SideBarIcon/>
                    </button>
                    //<!-- SVG logo -->
                    <div class="text-white text-lg font-bold">
                      //<!-- Insert your SVG logo here -->
                        <StacksIcon/>
                    </div>
                    //<!-- Text input -->
                    <input type="text" placeholder="Search" class="px-4 py-2 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-600"/>
                </div>
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