use leptos::prelude::*;
use oembed_rs::{find_provider, EmbedResponse};
use serde::{Serialize, de::DeserializeOwned};
use crate::icons::LinkIcon;
use crate::post::LinkType;

#[cfg(not(feature = "ssr"))]
pub fn fetch_api<T>(
    path: &str,
) -> impl std::future::Future<Output = Option<T>> + Send + '_
where
    T: Serialize + DeserializeOwned,
{
    use leptos::prelude::on_cleanup;
    use send_wrapper::SendWrapper;

    SendWrapper::new(async move {
        let abort_controller =
            SendWrapper::new(web_sys::AbortController::new().ok());
        let abort_signal = abort_controller.as_ref().map(|a| a.signal());

        // abort in-flight requests if, e.g., we've navigated away from this page
        on_cleanup(move || {
            if let Some(abort_controller) = abort_controller.take() {
                abort_controller.abort()
            }
        });

        gloo_net::http::Request::get(path)
            .abort_signal(abort_signal.as_ref())
            .send()
            .await
            .map_err(|e| log::error!("{e}"))
            .ok()?
            .json()
            .await
            .ok()
    })
}

#[cfg(feature = "ssr")]
pub async fn fetch_api<T>(path: &str) -> Option<T>
where
    T: Serialize + DeserializeOwned,
{
    reqwest::get(path)
        .await
        .map_err(|e| log::error!("{e}"))
        .ok()?
        .json()
        .await
        .ok()
}

/// Component to safely embed content at the url `link-input`.
/// It will try to infer the content type using the oembed API. If the provider of the url
/// is not in the whitelisted list of providers, it will instead try to naively embed the
/// content using the user-defined `link_input_type`
#[component]
pub fn Embed(
    #[prop(into)]
    link_input: Signal<String>,
    link_type_input: RwSignal<LinkType>,
) -> impl IntoView {
    let oembed_resource = Resource::new(
        move || link_input.get(),
        move |url| async move {
            match find_provider(&url) {
                Some((_provider, endpoint)) => {
                    let endpoint = format!("{}?url={url}", endpoint.url);
                    log::info!("Fetch oembed data: {endpoint}");
                    let oembed_data = fetch_api::<EmbedResponse>(&endpoint).await;
                    log::info!("{:?}", oembed_data);
                    oembed_data
                },
                None => None
            }
        },
    );

    view! {
        <Suspense>
        { match &*oembed_resource.read() {
            Some(oembed_data) => {
                log::info!("{:?}", oembed_data);
                view! {}.into_any()
            },
            None => view! { <NaiveEmbed link_input link_type_input/> }.into_any(),
        }}
        </Suspense>
    }
}

/// Component to naively and safely embed external content
#[component]
pub fn NaiveEmbed(
    #[prop(into)]
    link_input: Signal<String>,
    #[prop(into)]
    link_type_input: Signal<LinkType>,
) -> impl IntoView {
    // TODO decide if naively embed or just provide link?
    let media_class = "max-h-80 max-w-full object-contain";
    view! {
        { move || match link_type_input.get() {
            LinkType::None => None,
            LinkType::WebPage => Some(view! {
                <a href=link_input><LinkIcon/></a>
            }.into_any()),
            LinkType::Image => Some(view! {
                <img src=link_input class=media_class/>
            }.into_any()),
            // TODO check url to see if it has video extension
            LinkType::Video => Some(view! {
                <iframe src=link_input class=media_class sandbox></iframe>
            }.into_any()),
        }}
    }
}