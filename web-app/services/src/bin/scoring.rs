use app::{
    forum::get_all_forum_names
};

pub const DB_URL_ENV : &str = "DATABASE_URL";

#[tokio::main]
async fn main() {

    println!("Get forum names from DB");
    let forum_name_vec = get_all_forum_names().await.unwrap();

    println!("Got all forum names, here they are:");
    for forum_name in forum_name_vec {
        println!("Got forum: {forum_name}");
    }
}