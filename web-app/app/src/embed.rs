use std::collections::HashSet;
use ammonia::Builder;
use lazy_static::lazy_static;
use leptos::html;
use leptos::prelude::*;
use mime_guess::{from_path, mime};
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use strum_macros::{Display, EnumString, IntoStaticStr};
use url::Url;

#[cfg(feature = "ssr")]
use {
    reqwest::Client,
    http::header::{ACCEPT, USER_AGENT},
    http::{HeaderMap, HeaderValue},
};

use crate::errors::{AppError, ErrorDisplay};
use crate::icons::LinkIcon;

const DEFAULT_MEDIA_CLASS: &str = "h-fit w-fit max-h-160 max-w-full object-contain";
const THUMBNAIL_CLASS: &str = "h-16 w-16 object-contain";

lazy_static! {
    static ref PROVIDERS: Vec<OEmbedProvider> =
        serde_json::from_slice(include_bytes!("../embed/providers.json"))
            .expect("failed to load oEmbed providers");
}

#[derive(Clone, Copy, Debug, Default, Display, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum EmbedType {
    #[default]
    None = 0,
    Link = 1,
    Embed = 2,
}

#[repr(i16)]
#[derive(Clone, Copy, Debug, Default, Display, EnumString, Eq, IntoStaticStr, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub enum LinkType {
    #[default]
    None = -1,
    Link = 0,
    Image = 1,
    Video = 2,
    Rich = 3,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Link {
    pub link_type: LinkType,
    pub link_url: Option<String>,
    pub link_embed: Option<String>,
    pub link_thumbnail_url: Option<String>,
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
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "photo")]
    Photo(Photo),
    #[serde(rename = "video")]
    Video(Video),
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

impl From<i16> for LinkType {
    fn from(category_color_val: i16) -> Self {
        match category_color_val {
            x if x == LinkType::Link as i16 => LinkType::Link,
            x if x == LinkType::Image as i16 => LinkType::Image,
            x if x == LinkType::Video as i16 => LinkType::Video,
            x if x == LinkType::Rich as i16 => LinkType::Rich,
            _ => LinkType::None,
        }
    }
}

impl From<LinkType> for EmbedType {
    fn from(link_type: LinkType) -> Self {
        match link_type {
            LinkType::None => EmbedType::None,
            LinkType::Link => EmbedType::Link,
            _ => EmbedType::Embed,
        }
    }
}

impl Link  {
    pub fn new(
        link_type: LinkType,
        link_url: Option<String>,
        link_embed: Option<String>,
        link_thumbnail_url: Option<String>,
    ) -> Self {
        Self {
            link_type,
            link_url,
            link_embed,
            link_thumbnail_url,
        }
    }
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

#[server]
pub async fn get_oembed_data(url: String) -> Result<OEmbedReply, ServerFnError<AppError>> {
    let mut oembed_data = fetch_api::<OEmbedReply>(&url)
        .await
        .ok_or(AppError::new(format!("Cannot get oEmbed data at endpoint {url}")))?;
    
    match oembed_data.oembed_type {
        OEmbedType::Video(ref mut video) => video.html = clean_html(&video.html),
        OEmbedType::Rich(ref mut rich) => rich.html = clean_html(&rich.html),
        _ => ()
    };
    
    Ok(oembed_data)
}

/// Component to safely embed content at the url `link-input`.
/// It will try to infer the content type using the oembed API. If the provider of the url
/// is not in the whitelisted list of providers, it will instead try to naively embed the
/// content using file extension in the url and fallback to a simple link.
#[component]
pub fn EmbedPreview(
    embed_type_input: RwSignal<EmbedType>,
    #[prop(into)]
    link_input: Signal<String>,
    title_input: RwSignal<String>,
    select_ref: NodeRef<html::Select>,
) -> impl IntoView {
    // TODO try to simplify with new verify_link_and_get_embed function
    let link_resource = Resource::new(
        move || (embed_type_input.get(), link_input.get()),
        move |(embed_type, url)| async move {
            verify_link_and_get_embed(embed_type, &url).await
        },
    );

    view! {
        <Suspense>
        { move || link_resource.read().clone().map(|(link, title)| {
                title_input.update(|title_input| if title_input.is_empty() {
                    *title_input = title.unwrap_or_default();
                });
                select_embed_type(link.link_type, embed_type_input, select_ref);
                view! { <Embed link align_center=true/> }
            })
        }
        </Suspense>
    }
}

/// Component to safely embed external content
#[component]
pub fn Embed(
    link: Link,
    #[prop(default = false)]
    align_center: bool,
) -> impl IntoView {
    match (link.link_type, link.link_url, link.link_embed, link.link_thumbnail_url) {
        (LinkType::None, _, _, _) => None,
        (_, None, _, _) => None,
        (LinkType::Link, Some(link_url), None, thumbnail_url) => Url::parse(&link_url).ok().map(|url| view! {
            <LinkEmbed url thumbnail_url align_center/>
        }.into_any()),
        (link_type, Some(link_url), None, _) => Some(view! {
            <NaiveEmbed link_input=link_url link_type align_center/>
        }.into_any()),
        (_, Some(_), Some(link_embed), _) => Some(view! {
            <HtmlEmbed html=link_embed align_center/>
        }.into_any()),
    }
}

/// Component to naively and safely embed external content
#[component]
pub fn NaiveEmbed(
    #[prop(into)]
    link_input: Signal<String>,
    link_type: LinkType,
    #[prop(default = false)]
    align_center: bool,
) -> impl IntoView {
    view! {
        { move || {
            match (link_type, Url::parse(&link_input.read())) {
                (LinkType::None, _) => None,
                (_, Err(e)) => Some(view! { <ErrorDisplay error=AppError::new(format!("Invalid link: {e}"))/> }.into_any()),
                (LinkType::Link, Ok(url)) => Some(view! { <LinkEmbed url align_center/> }.into_any()),
                (LinkType::Image, Ok(url)) => Some(view! { <ImageEmbed url=url.to_string() align_center/> }.into_any()),
                (LinkType::Video, Ok(url)) => Some(view! { <VideoEmbed url=url.to_string() align_center/> }.into_any()),
                (LinkType::Rich, Ok(url)) => Some(view! { <LinkEmbed url align_center/> }.into_any()),
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
    #[prop(default = false)]
    align_center: bool,
) -> impl IntoView {
    match url.domain() {
        Some(domain) => {
            let class = match align_center {
                true => "flex justify-center items-center h-fit w-full",
                false => "flex justify-center 2xl:justify-start items-center h-fit w-full",
            };
            let clean_domain = match domain.starts_with("www.") {
                true => domain[4..].to_string(),
                false => domain.to_string(),
            };
            view! {
                <div class=class>
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
pub fn ImageEmbed(
    url: String,
    #[prop(default = false)]
    align_center: bool,
) -> impl IntoView {
    let class = match align_center {
        true => "flex justify-center items-center h-fit w-full",
        false => "flex justify-center 2xl:justify-start items-center h-fit w-full",
    };
    view! {
        <div class=class>
            <img src=url class=DEFAULT_MEDIA_CLASS/>
        </div>
    }
}

/// Component to embed a video
#[component]
pub fn VideoEmbed(
    url: String,
    #[prop(default = false)]
    align_center: bool,
) -> impl IntoView {
    let class = match align_center {
        true => "flex justify-center items-center h-fit w-full",
        false => "flex justify-center 2xl:justify-start items-center h-fit w-full",
    };
    view! {
        <div class=class>
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

/// Component to embed html
#[component]
pub fn HtmlEmbed(
    html: String,
    #[prop(default = false)]
    align_center: bool,
) -> impl IntoView {
    let class = match align_center {
        true => "flex justify-center items-center h-fit w-full",
        false => "flex justify-center 2xl:justify-start items-center h-fit w-full",
    };
    view! {
        <div class=class inner_html=html/>
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
    let client = Client::new();

    // Configure headers, as some api provider require them
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (compatible; RustApp/1.0)"),
    );

    client.get(path)
        .headers(headers)
        .send()
        .await
        .map_err(|e| log::error!("Request error: {e}"))
        .ok()?
        .json()
        .await
        .map_err(|e| log::error!("Deserialize error: {e}"))
        .ok()
}

/// Select the given `link_type` in the given `list_ref` node
fn select_embed_type(
    link_type: LinkType,
    link_embed: RwSignal<EmbedType>,
    select_ref: NodeRef<html::Select>
) {
    let new_embed_type = link_type.into();
    link_embed.update_untracked(|embed_type| *embed_type = new_embed_type);
    if let Some(select_ref) = select_ref.get_untracked() {
        select_ref.set_selected_index(new_embed_type as i32);
    };
}

/// Check that an url is valid and infer its type
pub fn check_url_and_infer_type(
    url: &Url,
) -> LinkType {
    if url.scheme() == "https" && url.domain().is_some() {
        let mime_guess = from_path(url.as_str());
        match mime_guess.first() {
            Some(mime_guess) if mime_guess.type_() == mime::IMAGE => LinkType::Image,
            Some(mime_guess) if mime_guess.type_() == mime::VIDEO => LinkType::Video,
            _ => LinkType::Link,
        }
    } else {
        LinkType::None
    }
}

fn filter_attribute_values(attribute_value: &str, allowed_values: &[&str]) -> String {
    attribute_value
        .split(';')
        .filter_map(|v| {
            let mut parts = v.split(':').map(str::trim);
            if let Some(key) = parts.next() {
                if allowed_values.contains(&key) {
                    return match parts.next() {
                        Some(value) => Some(format!("{}:{}", key, value)),
                        None => Some(key.to_string()),
                    };
                }
            }
            None
        })
        .collect::<Vec<String>>()
        .join(";")
}

pub fn clean_html(
    html: &str,
) -> String {
    log::debug!("Html: {}", html);

    // Create a set of allowed iframe attributes
    let iframe_attributes = HashSet::from(
        [
            "width",
            "height",
            "src",
            "frameborder",
            "allowfullscreen",
            "scrollbar",
            "allow",
            "style",
            "title",
        ]
    );

    let clean_html = Builder::default()
        .add_tags(["iframe"])
        .add_tag_attributes("iframe", iframe_attributes)
        .add_tag_attribute_values(
            "iframe",
            "referrerpolicy",
            ["strict-origin", "strict-origin-when-cross-origin"]
        ).attribute_filter(|element, attribute, value| {
        match (element, attribute) {
            ("iframe", "style") => {
                Some(filter_attribute_values(value, &["border", "min-width", "min-height, width, height"]).into())
            }
            ("iframe", "allow") => {
                Some(filter_attribute_values(value, &["encrypted-media", "picture-in-picture"]).into())
            }
            _ => Some(value.into())
        }
    })
        .clean(html)
        .to_string();
    log::debug!("clean_html: {}", clean_html);
    clean_html
}

/// Check the input `link`'s validity and returns Link and an optional title.
/// If embed_type is EmbedType::Link, the link is always embedded as a simple link,
/// otherwise the link type will be inferred using the oEmbed API or the file extension.
/// If the type cannot be inferred, it will fallback to a link.
pub async fn verify_link_and_get_embed(
    embed_type: EmbedType,
    link: &str,
) -> (Link, Option<String>) {
    match (embed_type, Url::parse(link)) {
        (_, Err(_)) => (Link::default(), None),
        (EmbedType::Link, Ok(url)) => (Link::new(LinkType::Link, Some(url.to_string()), None, None), None),
        (_, Ok(url)) => {
            match find_url_provider(url.as_str()) {
                Some((_provider, endpoint)) => {
                    // TODO check values for width and height
                    let endpoint = format!("{}?url={url}&maxwidth=800&maxheight=600", endpoint.url);
                    log::debug!("Fetch oembed data: {endpoint}");
                    match get_oembed_data(endpoint).await {
                        Ok(oembed_data) => {
                            let title = oembed_data.title;
                            let thumbnail_url = oembed_data.thumbnail_url;
                            let link = match oembed_data.oembed_type {
                                OEmbedType::Link => Link::new(LinkType::Link, Some(url.to_string()), None, thumbnail_url),
                                OEmbedType::Photo(photo) => Link::new(LinkType::Image, Some(photo.url), None, thumbnail_url),
                                OEmbedType::Video(video) => Link::new(LinkType::Video, Some(url.to_string()), Some(clean_html(&video.html)), thumbnail_url),
                                OEmbedType::Rich(rich) => Link::new(LinkType::Rich, Some(url.to_string()), Some(clean_html(&rich.html)), thumbnail_url),
                            };
                            (link, title)
                        },
                        Err(e) => {
                            log::debug!("Failed to get oembed data: {}", e);
                            let inferred_type = check_url_and_infer_type(&url);
                            let link = match inferred_type {
                                LinkType::None => None,
                                _ => Some(url.to_string()),
                            };
                            (Link::new(inferred_type, link, None, None), None)
                        },
                    }
                },
                None => {
                    let inferred_type = check_url_and_infer_type(&url);
                    let link = match inferred_type {
                        LinkType::None => None,
                        _ => Some(url.to_string()),
                    };
                    (Link::new(inferred_type, link, None, None), None)
                },
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use url::Url;
    use crate::embed::{check_url_and_infer_type, clean_html, find_url_provider, LinkType, OEmbedEndpoint, OEmbedProvider};

    #[test]
    fn test_link_type_from_i16() {
        assert_eq!(LinkType::from(-1), LinkType::None);
        assert_eq!(LinkType::from(0), LinkType::Link);
        assert_eq!(LinkType::from(1), LinkType::Image);
        assert_eq!(LinkType::from(2), LinkType::Video);
        assert_eq!(LinkType::from(3), LinkType::Rich);
        assert_eq!(LinkType::from(-2), LinkType::None);
        assert_eq!(LinkType::from(100), LinkType::None);
    }

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

    #[test]
    fn test_check_url_and_infer_type() {
        let no_domain_url = Url::parse("http://127.0.0.1:8000").expect("Should parse no_domain_url");
        let http_url = Url::parse("http://www.test.com/").expect("Should parse http_url");
        let https_link = Url::parse("https://www.test.com/").expect("Should parse https_link");
        let https_image = Url::parse("https://www.test.com/test.jpg").expect("Should parse https_image");
        let https_video = Url::parse("https://www.test.com/test.mp4").expect("Should parse https_video");
        assert_eq!(check_url_and_infer_type(&no_domain_url), LinkType::None);
        assert_eq!(check_url_and_infer_type(&http_url), LinkType::None);
        assert_eq!(check_url_and_infer_type(&https_link), LinkType::Link);
        assert_eq!(check_url_and_infer_type(&https_image), LinkType::Image);
        assert_eq!(check_url_and_infer_type(&https_video), LinkType::Video);
    }

    #[test]
    fn test_clean_html() {
        let input_html = r#"
            <div>Safe content</div>
            <iframe
                width="600"
                height="400"
                src="https://example.com/embed"
                style="border:1px solid black;min-width:300px;unsupported:invalid"
                allow="encrypted-media;picture-in-picture;autoplay"
                referrerpolicy="strict-origin"
                title="Example Embed">
            </iframe>
            <script>alert("malicious script");</script>
        "#;

        // Expected output after cleaning
        let expected_output = r#"
            <div>Safe content</div>
            <iframe width="600" height="400" src="https://example.com/embed" style="border:1px solid black;min-width:300px" allow="encrypted-media;picture-in-picture" referrerpolicy="strict-origin" title="Example Embed">
            </iframe>
        "#;

        assert_eq!(clean_html(input_html).trim(), expected_output.trim());

        let input_html = r#"
            <iframe
                src="https://example.com/embed"
                allow="accelerometer; gyroscope; clipboard-write;"
                referrerpolicy="unsafe-url">
            </iframe>
        "#;

        // Expected output after cleaning
        let expected_output = r#"
            <iframe src="https://example.com/embed" allow="">
            </iframe>
        "#;

        assert_eq!(clean_html(input_html).trim(), expected_output.trim());
    }
}