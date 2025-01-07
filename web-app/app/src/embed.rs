use lazy_static::lazy_static;
use leptos::html;
use leptos::prelude::*;
use mime_guess::{from_path, mime};
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use strum::IntoEnumIterator;
use url::Url;
use crate::errors::{AppError, ErrorDisplay};
use crate::icons::LinkIcon;
use crate::post::{LinkType};

const DEFAULT_MEDIA_CLASS: &str = "h-fit w-fit max-h-160 max-w-full object-contain";
const THUMBNAIL_CLASS: &str = "h-16 w-16 object-contain";

lazy_static! {
    static ref PROVIDERS: Vec<OEmbedProvider> =
        serde_json::from_slice(include_bytes!("../embed/providers.json"))
            .expect("failed to load oEmbed providers");
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct OEmbedProvider {
    pub provider_name: String,
    pub provider_url: String,
    pub endpoints: Vec<OEmbedEndpoint>,
}

/// Endpoint of oEmbed provider
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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
    link_type_input: RwSignal<LinkType>,
    select_ref: NodeRef<html::Select>
) {
    link_type_input.set(link_type);
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
    if let Ok(url) = Url::parse(url) {
        if url.scheme() == "https" && url.domain().is_some() {
            let mime_guess = from_path(url.as_str());
            match mime_guess.first() {
                Some(mime_guess) if mime_guess.type_() == mime::IMAGE => select_link_type(LinkType::Image, link_type, select_ref),
                Some(mime_guess) if mime_guess.type_() == mime::VIDEO => select_link_type(LinkType::Video, link_type, select_ref),
                _ => select_link_type(LinkType::Link, link_type, select_ref),
            };
        }
    } else {
        select_link_type(LinkType::None, link_type, select_ref);
    }
}

#[server]
pub async fn get_oembed_data(url: String) -> Result<OEmbedReply, ServerFnError<AppError>> {
    let oembed_data = fetch_api::<OEmbedReply>(&url)
        .await
        .ok_or(AppError::new(format!("Cannot get oEmbed data at endpoint {url}")))?;
    // TODO try to cleanup html with the ammonia crate
    Ok(oembed_data)
}

/// Component to safely embed content at the url `link-input`.
/// It will try to infer the content type using the oembed API. If the provider of the url
/// is not in the whitelisted list of providers, it will instead try to naively embed the
/// content using the user-defined `link_input_type`
#[component]
pub fn EmbedPreview(
    #[prop(into)]
    link_input: Signal<String>,
    link_type_input: RwSignal<LinkType>,
    title: RwSignal<String>,
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
                            Some(oembed_data) => Some((oembed_data, url)),
                            None => {
                                log::debug!("Failed to get oEmbed data in browser, try again through the server.");
                                get_oembed_data(endpoint).await.map_err(|e| log::error!("{e}")).ok().map(|data| (data, url))
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
            Some(Some((oembed_data, url))) => {
                title.update(|title| if title.is_empty() {
                    *title = oembed_data.title.clone().unwrap_or_default();
                });
                if let Some(select_ref) = select_ref.get_untracked() {
                    select_ref.set_disabled(true);
                };
                match &oembed_data.oembed_type {
                    OEmbedType::Link => {
                        select_link_type(LinkType::Link, link_type_input, select_ref);
                        match (Url::parse(url), oembed_data.thumbnail_url.clone()) {
                            (Ok(url), Some(thumbnail_url)) => Some(view! { <LinkEmbed url thumbnail_url=Some(thumbnail_url)/> }.into_any()),
                            (Ok(url), None) => Some(view! { <LinkEmbed url/> }.into_any()),
                            _ => Some(view! { <ErrorDisplay error=AppError::new("Invalid url")/> }.into_any()),
                        }
                    },
                    OEmbedType::Photo(photo) => {
                        select_link_type(LinkType::Image, link_type_input, select_ref);
                        Some(view! { <ImageEmbed url=photo.url.clone()/> }.into_any())
                    },
                    OEmbedType::Video(video) => {
                        select_link_type(LinkType::Video, link_type_input, select_ref);
                        Some(view! { <div class="flex justify-center items-center" inner_html=video.html.clone()/> }.into_any())
                    },
                    OEmbedType::Rich(rich) => {
                        select_link_type(LinkType::Rich, link_type_input, select_ref);
                        Some(view! { <div inner_html=rich.html.clone()/> }.into_any())
                    },
                }
            },
            Some(None) => {
                infer_link_type(&link_input.read(), link_type_input, select_ref);
                if let Some(select_ref) = select_ref.get_untracked() {
                    select_ref.set_disabled(false);
                };
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
    view! {
        { move || {
            match (link_type_input.get(), Url::parse(&link_input.read())) {
                (LinkType::None, _) => None,
                (_, Err(e)) => Some(view! { <ErrorDisplay error=AppError::new(format!("Invalid link: {e}"))/> }.into_any()),
                (LinkType::Link, Ok(url)) => Some(view! { <LinkEmbed url/> }.into_any()),
                (LinkType::Image, Ok(url)) => Some(view! { <ImageEmbed url=url.to_string()/> }.into_any()),
                (LinkType::Video, Ok(url)) => Some(view! { <VideoEmbed url=url.to_string()/> }.into_any()),
                (LinkType::Rich, Ok(url)) => Some(view! { <LinkEmbed url/> }.into_any()),
            }
        }}
    }
}

/// Component to embed a link with an optional thumbnail
#[component]
pub fn LinkEmbed(
    url: Url,
    #[prop(default = None)]
    thumbnail_url: Option<String>,
) -> impl IntoView {
    match url.domain() {
        Some(domain) => {
            let clean_domain = match domain.starts_with("www.") {
                true => domain[4..].to_string(),
                false => domain.to_string(),
            };
            view! {
                <div class="flex justify-center">
                    <a href=url.to_string() class="w-fit flex items-center gap-2 px-2 py-1 bg-primary rounded hover:bg-base-content/50">
                        { match thumbnail_url {
                            Some(thumbnail_url) => view! { <img src=thumbnail_url class=THUMBNAIL_CLASS/> }.into_any(),
                            None => view! { <LinkIcon/> }.into_any(),
                        }}
                        <div>{clean_domain}</div>
                    </a>
                </div>
            }.into_any()
        },
        None => view! { <ErrorDisplay error=AppError::new("Invalid domain name")/> }.into_any(),
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

/// Component to embed a video
#[component]
pub fn VideoEmbed(url: String) -> impl IntoView {
    view! {
        <div class="flex justify-center items-center">
            <video
                src=url
                class=DEFAULT_MEDIA_CLASS
                controls
            >
                "Your browser doesn't support this video's format."
            </video>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::embed::{find_url_provider, OEmbedEndpoint, OEmbedProvider};

    #[test]
    fn test_oembed_provider_find_matching_endpoint() {
        let endpoint1 = OEmbedEndpoint {
            schemes: vec![
                String::from("https://www.test.com/image/*"),
                String::from("https://www.test.com/profile/*/post/*"),
            ],
            url: "https://www.test.com/embed".to_string(),
            discovery: false,
        };

        let endpoint2 = OEmbedEndpoint {
            schemes: vec![
                String::from("https://www.test.com/video/*"),
            ],
            url: "https://www.test.com/embed".to_string(),
            discovery: false,
        };

        let provider = OEmbedProvider {
            provider_name: "test".to_string(),
            provider_url: "https://www.test.com".to_string(),
            endpoints: vec![endpoint1.clone(), endpoint2.clone()],
        };

        assert_eq!(provider.find_matching_endpoint("https://www.test.com/image/a"), Some(&endpoint1));
        assert_eq!(provider.find_matching_endpoint("https://www.test.com/profile/1/post/a"), Some(&endpoint1));
        assert_eq!(provider.find_matching_endpoint("https://www.test.com/video/a"), Some(&endpoint2));
        assert_eq!(provider.find_matching_endpoint("https://www.other.com/profile/1/post/a"), None);
    }

    #[test]
    fn test_oembed_endpoint_has_matching_scheme() {
        let endpoint = OEmbedEndpoint {
            schemes: vec![
                String::from("https://www.test.com/image/*"),
                String::from("https://www.test.com/profile/*/post/*"),
            ],
            url: "https://www.test.com/embed".to_string(),
            discovery: false,
        };

        assert_eq!(endpoint.has_matching_scheme("https://www.test.com/image/a"), true);
        assert_eq!(endpoint.has_matching_scheme("https://www.test.com/profile/1/post/a"), true);
        assert_eq!(endpoint.has_matching_scheme("https://www.test.com/profile/1/posts/a"), false);
        assert_eq!(endpoint.has_matching_scheme("https://www.other.com/profile/1/post/a"), false);
    }

    #[test]
    fn test_find_url_provider() {
        let (youtube_provider, youtube_endpoint) = find_url_provider("https://www.youtube.com/watch?v=test").expect("Find youtube provider");
        assert_eq!(youtube_provider.provider_name, String::from("YouTube"));
        assert_eq!(youtube_provider.provider_url, String::from("https://www.youtube.com/"));
        assert_eq!(youtube_endpoint.url, String::from("https://www.youtube.com/oembed"));
        assert_eq!(youtube_endpoint.discovery, true);

        let (giphy_provider, giphy_endpoint) = find_url_provider("https://giphy.com/gifs/test").expect("Find giphy provider");
        assert_eq!(giphy_provider.provider_name, String::from("GIPHY"));
        assert_eq!(giphy_provider.provider_url, String::from("https://giphy.com"));
        assert_eq!(giphy_endpoint.url, String::from("https://giphy.com/services/oembed"));
        assert_eq!(giphy_endpoint.discovery, true);
    }
}