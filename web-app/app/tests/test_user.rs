use std::collections::BTreeSet;

use app::errors::AppError;
use app::user;

use crate::common::{create_user, get_db_pool};

mod common;

#[tokio::test]
async fn test_get_matching_username_set() -> Result<(), AppError> {
    let db_pool = get_db_pool().await;

    let num_users = 20usize;
    let mut expected_username_set = BTreeSet::<String>::new();
    for i in 0..num_users {
        expected_username_set.insert(
            create_user(
                i.to_string().as_str(),
                &db_pool,
            )
                .await
                .username
        );
    }

    let username_set = user::ssr::get_matching_username_set("1", num_users as i64, &db_pool).await?;

    let mut previous_username = None;
    for username in username_set {
        assert_eq!(username.chars().next().unwrap(), '1');
        if let Some(previous_username) = previous_username {
            assert!(previous_username < username)
        }
        previous_username = Some(username);
    }

    for i in num_users..2 * num_users {
        expected_username_set.insert(
            create_user(
                i.to_string().as_str(),
                &db_pool,
            )
                .await
                .username
        );
    }

    let username_set = user::ssr::get_matching_username_set("", num_users as i64, &db_pool).await?;

    assert_eq!(username_set.len(), num_users);

    Ok(())
}
