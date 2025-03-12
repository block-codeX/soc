use crate::models::User;
use mongodb::{Client, Collection};
use std::sync::Arc;

pub type Db = Arc<Collection<User>>;

pub async fn connect() -> Db {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("failed to connect to mongodb");

    let db = client.database("soc");
    Arc::new(db.collection::<User>("users"))
}
