

use crate::models::{User, Event};
use mongodb::{Client, Collection};
use std::sync::Arc;
use dotenv::dotenv;
use std::env;



// pub type Db<T> = Arc<Collection<T>>;

pub async fn connect<T>() -> Collection<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + Unpin + 'static,
{
    dotenv().ok();

    let client = Client::with_uri_str(env::var("DATABASE_URL").expect("Database connection not set"))
        .await
        .expect("failed to connect to mongodb");

    let db = client.database("soc");
    // Arc::new(db.collection::<T>("users"))
    db.collection(std::any::type_name::<T>().split("::").last().unwrap())
}
