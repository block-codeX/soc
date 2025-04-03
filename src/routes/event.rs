use crate::models::event::EventType;
use crate::models::{Attendee, Event, User};
use futures::{StreamExt, TryStreamExt};
use mongodb::Cursor;
use mongodb::{bson::{doc, oid::{self, ObjectId}}, options::FindOptions, Collection};
use rocket::{post, serde::json::Json, State};
use rocket::http::Status;
use serde::{Deserialize, Serialize};

use super::auth::{AdminUser, AuthToken};
use super::AuthenticatedUser;

const PINNED_EVENT :bool  = false;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJoinRequest {
    pub user_id: String,  // We'll parse this to ObjectId in the handler
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventRequest {
    name: String,
    location: String,
    date: String,
    host_id: Option<ObjectId>,
    description: String,
    event_type: EventType
}

#[post("/event", format="json", data="<new_event>")]
pub async  fn create_event(new_event: Json<EventRequest>,
     Database: &State<Collection<Event>>,
     _admin: AdminUser, // only admin can call this
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user
    ) -> Json<String> {
    // ensure the id is a valid obj
    
    let new_event = Event {
        id: None,
        name: new_event.name.clone(),
        location: new_event.location.clone(),
        date: new_event.date.clone(),
        host_id: new_event.host_id,
        event_type: new_event.event_type.clone(),
        description: new_event.description.clone(),
        attendees: vec![],
        image_url: Some("".to_string()),
        pinned: PINNED_EVENT
    };
    let result  = Database.insert_one(new_event).await;

    match result {
        Ok(_) => Json("Event successfully created".to_string()),
        Err(_) => Json(Status::InternalServerError.to_string())
    }
}

#[get("/event/<event_id>")]
pub async  fn read_event(db: &State<Collection<Event>>, 
    event_id: &str,
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user
) -> Result<Json<Event>, Status> {
    let collection = db;
    let object_id = match ObjectId::parse_str(event_id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest)
    };
    let filter = doc! {"_id": object_id};
    let result = collection.find_one(filter).await;
    eprintln!("loging...{:?}",result);
    match result {
        Ok(fetch_data) => {
            if let Some(data) = fetch_data {
                Ok(Json(data))
            } else {
                Err(Status::NotFound)
            }
        }
        Err(_) => Err(Status::InternalServerError)
    }
}

#[get("/events")]
pub async fn read_events(Database: &State<Collection<Event>>,
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user
) -> Json<Vec<Event>> {
    let mut cursor :Cursor<Event> = Database
        .find( doc! {})
        .await
        .expect("Failed to find evets");
    let mut events: Vec<Event> = Vec::new();
    while let Some(event) = cursor.try_next().await.expect("Error iterating cursor") {
        events.push(event);
    }
    Json(events)
}

#[put("/event/<event_id>", format = "json", data = "<updated_event>")]
pub async fn update_event(
    event_id: &str,
    updated_event: Json<Event>,
    db: &State<Collection<Event>>,
    _admin: AdminUser, // only admin can call this
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user
) -> Result<Json<String>, Status> {
    let collection = db;

    // Convert event_id to ObjectId
    let event_oid = match ObjectId::parse_str(event_id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest) // Return 400 if it's not a valid ObjectId
    };

    let  update_doc = doc! {
        "$set" : {
            "name": &updated_event.name,
        "location": &updated_event.location,
        "date": &updated_event.date,
        }
    };
    
    let filter = doc! {"_id": event_oid};
    
    match collection
    .find_one_and_update(filter, update_doc).await {
        Ok(Some(_)) => Ok(Json("User successfully  updated".to_string())),
        Ok(None)=> {
            
            Err(Status::NotFound)
        },
        Err(_e)=> {
            Err(Status::InternalServerError)
        }
    }
    
}

#[delete("/event/<event_id>")]
pub async fn drop_event(event_id: &str, 
    db: &State<Collection<Event>>,
    _admin: AdminUser, // only admin can call this
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user           
) -> Result<Json<String>, Status> {
    let collection = db;

    let object_id = match ObjectId::parse_str(event_id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest)
    };
    let filter = doc! {"_id": object_id};
    let result = collection.delete_one(filter).await;

    match result {
       Ok(deleted_result) => {
        if deleted_result.deleted_count > 0 {
            Ok(Json("User deleted successfully".to_string()))
        }else {
            Err(Status::NotFound)
        }
       }
       Err(_)=> Err(Status::InternalServerError)
    }
}

#[put("/event/join/<event_id>", format = "json", data = "<user_data>")]
pub async fn join_event(
    event_id: &str,
    user_data: Json<UserJoinRequest>,
    db: &State<Collection<Event>>,
    user_collection: &State<Collection<User>>,
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user     
) -> Result<Json<String>, Status> {
    // Convert event_id and user_id to ObjectId
    let event_oid = match ObjectId::parse_str(event_id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest), // Invalid event ID
    };

    let user_oid = match ObjectId::parse_str(&user_data.user_id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest), // Invalid user ID
    };

    // Create the new Attendee object
    let new_attendee = Attendee {
        user_id: user_oid,
        name: user_data.name.clone(),
        email: user_data.email.clone(),
    };

    // Update event by adding the new Attendee to the attendees list
    let update_doc = doc! {
        "$addToSet": { // $addToSet ensures no duplicates
            "attendees": new_attendee
        }
    };

    let filter = doc! {"_id": event_oid};

    let event_result =  db.find_one_and_update(filter.clone(), update_doc).await;

    if let Err(_) = event_result {
        return Err(Status::InternalServerError);
    }

    let update_user = doc! {
        "$addToSet": {"attending_events": event_oid} // ensure no duplicates
    };
    let user_filter = doc! {"_id": user_oid};

    let user_result = user_collection.update_one(user_filter.clone(), update_user).await;

    if let Err(_) = user_result {
        // Rollback changes if failure
        let rollback_event = doc! {"$pull": { "attendees": user_oid }};
        let _ = db.update_one(filter, rollback_event).await;
        return Err(Status::InternalServerError);
    }

    Ok(Json("Successfully Join the event".to_string()))
}

#[delete("/event/leave/<event_id>/<user_id>")]
pub async fn leave_event(
    event_id: &str,
    user_id: &str,
    db: &State<Collection<Event>>,
    user_collection: &State<Collection<User>>,
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user     
) -> Result<Json<String>, Status> {
    // Convert event_id and user_id to ObjectId
    let event_oid = match ObjectId::parse_str(event_id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest), // Invalid event ID
    };

    let user_oid = match ObjectId::parse_str(user_id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest), // Invalid user ID
    };

    // Update event by removing the attendee with matching user_id
    let update_doc = doc! {
        "$pull": {
            "attendees": {
                "user_id": user_oid
            }
        }
    };

    let filter = doc! {"_id": event_oid};

    let event_result = db.find_one_and_update(filter.clone(), update_doc).await;
    if let Err(_) = event_result {
        return Err(Status::InternalServerError);
    }

    let update_user = doc ! {
        "$pull": {"attending_events": event_oid} // remove event from the user attending list
    };
    let user_filter = doc! {"_id": user_oid};
    let user_result = user_collection.update_one(user_filter.clone(), update_user).await;
    if let Err(_) = user_result {
        // rollback event incase of failure
        let rollback_event = doc! {"$addToSet": {"attendees": user_oid}};
        let _ = db.update_one(filter, rollback_event).await;
        return Err(Status::InternalServerError);
    }

    Ok(Json("Successfully left the event.".to_string()))
}

#[post("/events/multiple", format = "json", data = "<event_ids>")]
pub async fn get_multiple_events(
    event_ids: Json<Vec<String>>, // Accepts a JSON array of event IDs
    db: &State<Collection<Event>>,
    _token: AuthToken, // verfiy blacklisted tokens
    _user: AuthenticatedUser // verity authenticated user     
) -> Result<Json<Vec<Event>>, Status> {
    // Convert string IDs to ObjectId
    let object_ids: Vec<ObjectId> = event_ids
        .iter()
        .filter_map(|id| ObjectId::parse_str(id).ok()) // Ignore invalid IDs
        .collect();

    if object_ids.is_empty() {
        return Err(Status::BadRequest); // Return error if no valid ObjectIds
    }

    // Query to find all events where `_id` is in the provided list
    let filter = doc! { "_id": { "$in": &object_ids } };

    let mut cursor = match db.find(filter).await {
        Ok(cursor) => cursor,
        Err(_) => return Err(Status::InternalServerError),
    };

    let mut events = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(event) => events.push(event),
            Err(_) => return Err(Status::InternalServerError),
        }
    }

    Ok(Json(events))
}
