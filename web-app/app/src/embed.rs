use std::collections::HashMap;
use lazy_static::lazy_static;
use leptos::html;
use leptos::prelude::*;
use oembed_rs::{matches_scheme, EmbedType, Endpoint, Provider};
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use serde_json::Value;
use strum::IntoEnumIterator;

use crate::icons::LinkIcon;
use crate::post::LinkType;

const DEFAULT_MEDIA_CLASS: &str = "h-fit w-fit max-h-160 max-w-full object-contain";

/// oEmbed response with optional version attribute
/// Used to handle providers that don't respect the specification
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct OptionalVersionEmbedResponse {
    #[serde(flatten)]
    pub oembed_type: EmbedType,
    pub version: Option<String>,
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub provider_name: Option<String>,
    pub provider_url: Option<String>,
    pub cache_age: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

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

        let reply = gloo_net::http::Request::get(path)
            .abort_signal(abort_signal.as_ref())
            .send()
            .await;

        log::info!("Reply: {:?}", reply);

        reply.map_err(|e| log::error!("API error {e}"))
            .ok()?
            .json()
            .await
            .map_err(|e| log::error!("Deserialize error {e}"))
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
        .map_err(|e| log::error!("API error {e}"))
        .ok()?
        .json()
        .await
        .map_err(|e| log::error!("Deserialize error {e}"))
        .ok()
}

lazy_static! {
    static ref PROVIDERS: Vec<Provider> =
        serde_json::from_slice(include_bytes!("../embed/providers.json"))
            .expect("failed to load oEmbed providers");
}

/// Find the oEmbed provider and endpoint based on the URL using ShareSphere's providers.json
pub fn find_provider(url: &str) -> Option<(&Provider, &Endpoint)> {
    PROVIDERS.iter().find_map(|p| {
        p.endpoints
            .iter()
            .find(|e| e.schemes.iter().any(|s| matches_scheme(s, url)))
            .map(|e| (p, e))
    })
}

/// Select the given `link_type` in the given `list_ref` node
pub fn select_link_type(
    link_type: LinkType,
    select_ref: NodeRef<html::Select>
) {
    match select_ref.get_untracked() {
        Some(select_ref) => {
            let index = LinkType::iter().position(|variant| variant == link_type);
            if let Some(index) = index {
                select_ref.set_selected_index(index as i32);
            }
        },
        None => log::error!("Link type select failed to load."),
    };
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
    select_ref: NodeRef<html::Select>,
) -> impl IntoView {
    let oembed_resource = Resource::new(
        move || link_input.get(),
        move |url| async move {
            match url.is_empty() {
                true => None,
                false => match find_provider(&url) {
                    Some((_provider, endpoint)) => {
                        let endpoint = format!("{}?url={url}&maxwidth=800&maxheight=600", endpoint.url);
                        log::info!("Fetch oembed data: {endpoint}");
                        // TODO: add server-side backup in case CORS is missing
                        let oembed_data = fetch_api::<OptionalVersionEmbedResponse>(&endpoint).await;
                        log::info!("{:?}", oembed_data);
                        oembed_data
                    },
                    None => {
                        log::info!("Could not find oembed provider for url: {url}");
                        None
                    }
                }
            }
        },
    );

    view! {
        <Suspense>
        { move || match &*oembed_resource.read() {
            Some(Some(oembed_data)) => {
                log::info!("Embed data {:?}", oembed_data);
                // TODO automatically set title?
                match &oembed_data.oembed_type {
                    EmbedType::Link => {
                        link_type_input.set(LinkType::Link);
                        select_link_type(LinkType::Link, select_ref);
                        Some(view! { <a href=link_input><LinkIcon/></a> }.into_any())
                    },
                    EmbedType::Photo(photo) => {
                        link_type_input.set(LinkType::Image);
                        select_link_type(LinkType::Image, select_ref);
                        Some(view! { <ImageEmbed url=photo.url.clone()/> }.into_any())
                    },
                    EmbedType::Video(video) => {
                        link_type_input.set(LinkType::Video);
                        select_link_type(LinkType::Video, select_ref);
                        Some(view! { <div inner_html=video.html.clone()/> }.into_any())
                    },
                    EmbedType::Rich(rich) => {
                        link_type_input.set(LinkType::Rich);
                        select_link_type(LinkType::Rich, select_ref);
                        Some(view! { <div inner_html=rich.html.clone()/> }.into_any())
                    },
                }
            },
            Some(None) => Some(view! { <NaiveEmbed link_input link_type_input/> }.into_any()),
            None => None
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
    // TODO check link for extension to decide format
    view! {
        { move || match link_type_input.get() {
            LinkType::None => None,
            LinkType::Link => Some(view! { <a href=link_input><LinkIcon/></a> }.into_any()),
            LinkType::Image => Some(view! { <ImageEmbed url=link_input.get()/> }.into_any()),
            // TODO check url to see if it has video extension
            LinkType::Video => Some(view! { <iframe src=link_input class=DEFAULT_MEDIA_CLASS></iframe> }.into_any()),
            LinkType::Rich => Some(view! { <a href=link_input><LinkIcon/></a> }.into_any()),
        }}
    }
}

/// Component to embed an image
#[component]
pub fn ImageEmbed(url: String) -> impl IntoView {
    view! {
        <div class="flex justify-center items-center">
            <img src=url class=DEFAULT_MEDIA_CLASS/>
        </div>
    }
}