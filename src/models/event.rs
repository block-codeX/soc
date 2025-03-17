use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,

    pub name: String,
    #[serde(default)] // if location is missing use default value
    pub location: String,
    #[serde(default)]
    pub date: String,

    #[serde(rename = "user_id")]
    pub user_id: ObjectId
}