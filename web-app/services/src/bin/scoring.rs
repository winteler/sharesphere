use app::{
    post::ssr::{update_post_scores}
};

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");
    log::info!("Recompute posts rankings, trigger the refresh of their generated columns");
    let result = update_post_scores().await;
    if let Err(e) = result {
        log::error!("Failed to updated posts' ranking timestamps with error: {e}");
    } else {
        log::info!("Successfully updated posts' ranking timestamps");
    }
}