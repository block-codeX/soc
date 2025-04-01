use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::Attendee;


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Hackathon,
    Meetup,
    Workshop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,

    pub name: String,
    #[serde(default)] // if location is missing use default value
    pub location: String,
    #[serde(default)]
    pub date: String,

    pub description: String,
    pub event_type: EventType,



    #[serde(rename = "host_id", skip_serializing_if = "Option::is_none")]
    pub host_id: Option<ObjectId>,

    #[serde(default)]
    pub attendees: Vec<Attendee>,
}