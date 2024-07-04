use app::app::ssr::get_db_pool;
use app::post::ssr::update_post_scores;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("Should be able to initialize logging");
    let db_pool = get_db_pool().expect("Should be able to get db pool.");
    log::info!("Recompute posts rankings, trigger the refresh of their generated columns");
    let result = update_post_scores(&db_pool).await;
    if let Err(e) = result {
        log::error!("Failed to updated posts' ranking timestamps with error: {e}");
    } else {
        log::info!("Successfully updated posts' ranking timestamps");
    }
}