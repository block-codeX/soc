#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,

    pub name: String,
    pub location: String,
    pub date: String,

    #[serde(rename = "userId")]
    pub user_id: ObjectId
}