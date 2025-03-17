use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Application {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub event_id: ObjectId,
    pub status: ApplicationStatus
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")] // Stores as lowercase strings in MongoDB
pub enum ApplicationStatus {
    Pending,
    Accepted,
    Rejected,
}