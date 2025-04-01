use mongodb::bson::{self, doc, oid::{self, ObjectId}, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attendee {
    pub user_id: ObjectId,
    pub name: String,
    pub email: String
}

impl From<Attendee> for Bson {
    fn from(attendee: Attendee) -> Self {
        // Convert Attendee struct to a BSON document (a map of key-value pairs)
        Bson::Document(doc! {
            "user_id": attendee.user_id,
            "name": attendee.name,
            "email": attendee.email,
        })
    }
}