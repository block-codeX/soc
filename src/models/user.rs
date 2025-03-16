use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub wallet: String,

    #[serde(rename = "user_rank", skip_serializing_if = "Option::is_none")]
    pub user_rank: Option<i32>,
}
