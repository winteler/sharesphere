use std::collections::BTreeSet;
use url::Url;

fn main() {
    const OEMBED_PROVIDER_JSON_PATH: &str = "../../core/content/embed/oembed_providers.json";
    let data = std::fs::read_to_string(OEMBED_PROVIDER_JSON_PATH).expect("Should find OEmbed providers json.");
    let providers: serde_json::Value = serde_json::from_str(&data).unwrap();
    let provider_vec = providers.as_array().unwrap();

    let connect_origins: BTreeSet<String> = provider_vec
        .iter()
        .flat_map(|p| p["endpoints"].as_array().unwrap())
        .filter_map(|e| e["url"].as_str())
        .filter_map(|url| Url::parse(url).ok())
        .filter_map(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
        .collect();

    let frame_origins: BTreeSet<String> = provider_vec
        .iter()
        .flat_map(|p| p["endpoints"].as_array().unwrap())
        .filter_map(|e| e["schemes"].as_array())
        .flatten()
        .filter_map(|s| s.as_str())
        .filter_map(|s| Url::parse(s).ok())
        .filter_map(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
        .collect();

    println!("cargo:rustc-env=OEMBED_CONNECT_SRC={}", connect_origins.into_iter().collect::<Vec<_>>().join(" "));
    println!("cargo:rustc-env=OEMBED_FRAME_SRC={}", frame_origins.into_iter().collect::<Vec<_>>().join(" "));
    println!("cargo:rerun-if-changed={OEMBED_PROVIDER_JSON_PATH}");
}