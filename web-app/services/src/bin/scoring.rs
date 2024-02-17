use app::{
    post::ssr::get_post_to_rank_vec
};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {

    let mut join_set: JoinSet<Result<(), anyhow::Error>> = JoinSet::new();

    println!("Get posts to rank");
    let post_to_rank_vec = get_post_to_rank_vec().await.unwrap();

    println!("Got all posts to rank, update their score.");
    for post in post_to_rank_vec {
        join_set.spawn(async move {
            log::info!("Update score of post: {}", post.post_id);

        });
    }

    while let Some(result) = join_set.join_next().await {
        if let Err(error) = result {
            println!("Failed to update score of post with error: {error}");
        }
    }
}