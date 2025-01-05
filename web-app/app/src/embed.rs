use lazy_static::lazy_static;
use leptos::html;
use leptos::prelude::*;
use mime_guess::{from_path, mime};
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use strum::IntoEnumIterator;
use crate::errors::AppError;
use crate::icons::LinkIcon;
use crate::post::{LinkType};

const DEFAULT_MEDIA_CLASS: &str = "h-fit w-fit max-h-160 max-w-full object-contain";

lazy_static! {
    static ref PROVIDERS: Vec<OEmbedProvider> =
        serde_json::from_slice(include_bytes!("../embed/providers.json"))
            .expect("failed to load oEmbed providers");
}

#[derive(Debug, Deserialize)]
pub struct OEmbedProvider {
    pub provider_name: String,
    pub provider_url: String,
    pub endpoints: Vec<OEmbedEndpoint>,
}

/// Endpoint of oEmbed provider
#[derive(Debug, Deserialize)]
pub struct OEmbedEndpoint {
    #[serde(default)]
    pub schemes: Vec<String>,
    pub url: String,
    #[serde(default)]
    pub discovery: bool,
}

/// oEmbed type, as defined in section 2.3.4 of the [oEmbed specification][1].
///
/// [1]: https://oembed.com/
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum OEmbedType {
    #[serde(rename = "photo")]
    Photo(Photo),
    #[serde(rename = "video")]
    Video(Video),
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "rich")]
    Rich(Rich),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Video {
    pub html: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Photo {
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rich {
    pub html: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

/// oEmbed reply
/// Set version as optional to handle providers that don't respect the specification
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct OEmbedReply {
    #[serde(flatten)]
    pub oembed_type: OEmbedType,
    pub version: Option<String>,
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub provider_name: Option<String>,
    pub provider_url: Option<String>,
    pub cache_age: Option<i32>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
}

impl OEmbedProvider {
    /// Find an endpoint with one scheme matching the input `url` for this provider
    pub fn find_matching_endpoint(&self, url: &str) -> Option<&OEmbedEndpoint> {
        self.endpoints.iter().find(|&endpoint| endpoint.has_matching_scheme(url))
    }
}

impl OEmbedEndpoint {
    /// Find a scheme matching the input `url` for this endpoint
    pub fn has_matching_scheme(&self, url: &str) -> bool {
        self.schemes.iter().any(|scheme| url_matches_scheme(url, scheme))
    }
}

/// # Check if the `scheme` matches the given `url`
///
/// ```
/// use crate::app::embed::url_matches_scheme;
///
/// assert_eq!(url_matches_scheme("https://www.youtube.com/watch?v=test", "https://*.youtube.com/watch*"), true);
/// assert_eq!(url_matches_scheme("https://bsky.app/profile/test/post/testpost", "https://bsky.app/profile/*/post/*"), true);
/// assert_eq!(url_matches_scheme("https://bsky.app/profile/test", "https://*.youtube.com/watch*"), false);
/// ```
pub fn url_matches_scheme(mut url: &str, scheme: &str) -> bool {
    for (i, pattern) in scheme.split('*').enumerate() {
        if pattern.is_empty() {
            continue;
        }

        if let Some(index) = url.find(pattern) {
            if i == 0 && index > 0 {
                // the url should start with the first pattern
                return false;
            }
            url = &url[(index + pattern.len())..];
        } else {
            return false;
        }
    }
    scheme.ends_with('*') || url.is_empty()
}

/// Find the oEmbed provider and endpoint based on the URL using ShareSphere's providers.json
pub fn find_url_provider(url: &str) -> Option<(&OEmbedProvider, &OEmbedEndpoint)> {
    PROVIDERS.iter().find_map(|provider| {
        provider.find_matching_endpoint(url).map(|endpoint| (provider, endpoint))
    })
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

        gloo_net::http::Request::get(path)
            .abort_signal(abort_signal.as_ref())
            .send()
            .await
            .map_err(|e| log::error!("API error {e}"))
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
        .map_err(|e| log::error!("Request error: {e}"))
        .ok()?
        .json()
        .await
        .map_err(|e| log::error!("Deserialize error: {e}"))
        .ok()
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

/// Infer a link's type based on the url and update the link type signal based on the result
pub fn infer_link_type(
    url: &str,
    link_type: RwSignal<LinkType>,
    select_ref: NodeRef<html::Select>
) {
    let mime_guess = from_path(url);
    match mime_guess.first() {
        Some(mime_guess) if mime_guess.type_() == mime::IMAGE => {
            link_type.set(LinkType::Image);
            select_link_type(LinkType::Image, select_ref);
        },
        Some(mime_guess) if mime_guess.type_() == mime::VIDEO => {
            link_type.set(LinkType::Video);
            select_link_type(LinkType::Video, select_ref);
        },
        _ => (),
    }
}

#[server]
pub async fn get_oembed_data(url: String) -> Result<OEmbedReply, ServerFnError<AppError>> {
    let oembed_data = fetch_api::<OEmbedReply>(&url)
        .await
        .ok_or(AppError::new("Cannot get oEmbed data at endpoint {url}"))?;
    Ok(oembed_data)
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
                false => match find_url_provider(&url) {
                    Some((_provider, endpoint)) => {
                        // TODO check values for width and height
                        let endpoint = format!("{}?url={url}&maxwidth=800&maxheight=600", endpoint.url);
                        log::debug!("Fetch oembed data: {endpoint}");
                        match fetch_api::<OEmbedReply>(&endpoint).await {
                            Some(oembed_data) => Some(oembed_data),
                            None => {
                                log::debug!("Failed to get oEmbed data in browser, try again through the server.");
                                get_oembed_data(endpoint).await.map_err(|e| log::error!("{e}")).ok()
                            }
                        }
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
                // TODO automatically set title?
                match &oembed_data.oembed_type {
                    OEmbedType::Link => {
                        link_type_input.set(LinkType::Link);
                        select_link_type(LinkType::Link, select_ref);
                        Some(view! { <a href=link_input><LinkIcon/></a> }.into_any())
                    },
                    OEmbedType::Photo(photo) => {
                        link_type_input.set(LinkType::Image);
                        select_link_type(LinkType::Image, select_ref);
                        Some(view! { <ImageEmbed url=photo.url.clone()/> }.into_any())
                    },
                    OEmbedType::Video(video) => {
                        link_type_input.set(LinkType::Video);
                        select_link_type(LinkType::Video, select_ref);
                        Some(view! { <div inner_html=video.html.clone()/> }.into_any())
                    },
                    OEmbedType::Rich(rich) => {
                        link_type_input.set(LinkType::Rich);
                        select_link_type(LinkType::Rich, select_ref);
                        Some(view! { <div inner_html=rich.html.clone()/> }.into_any())
                    },
                }
            },
            Some(None) => {
                infer_link_type(&*link_input.read(), link_type_input, select_ref);
                Some(view! { <NaiveEmbed link_input link_type_input/> }.into_any())
            },
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
        { move || {
            match link_type_input.get() {
                LinkType::None => None,
                LinkType::Link => Some(view! { <a href=link_input><LinkIcon/></a> }.into_any()),
                LinkType::Image => Some(view! { <ImageEmbed url=link_input.get()/> }.into_any()),
                // TODO check url to see if it has video extension
                LinkType::Video => Some(view! {
                    <iframe
                        src=link_input
                        class=DEFAULT_MEDIA_CLASS
                        sandbox="allow-scripts allow-same-origin"
                        allow="autoplay"
                        referrerpolicy="no-referrer">
                    ></iframe>
                }.into_any()),
                LinkType::Rich => Some(view! { <a href=link_input><LinkIcon/></a> }.into_any()),
            }
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