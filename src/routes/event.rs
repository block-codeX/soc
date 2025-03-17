use crate::models::event::EventType;
use crate::models::Event;
use futures::TryStreamExt;
use mongodb::Cursor;
use mongodb::{bson::{doc, oid::{self, ObjectId}}, options::FindOptions, Collection};
use rocket::{post, serde::json::Json, State};
use rocket::http::Status;

#[post("/event", format="json", data="<new_event>")]
pub async  fn create_event(new_event: Json<Event>, Database: &State<Collection<Event>>) -> Json<String> {
    // ensure the id is a valid obj
    
    let new_event = Event {
        id: None,
        name: new_event.name.clone(),
        location: new_event.location.clone(),
        date: new_event.date.clone(),
        host_id: new_event.host_id,
        event_type: new_event.event_type.clone(),
        description: new_event.description.clone()
    };
    let result  = Database.insert_one(new_event).await;

    match result {
        Ok(_) => Json("Event successfully created".to_string()),
        Err(_) => Json(Status::InternalServerError.to_string())
    }
}

#[get("/event/<event_id>")]
pub async  fn read_event(db: &State<Collection<Event>>, event_id: &str) -> Result<Json<Event>, Status> {
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
pub async fn read_events(Database: &State<Collection<Event>>) -> Json<Vec<Event>> {
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
    db: &State<Collection<Event>>
) -> Result<Json<String>, Status> {
    let collection = db;

    let  update_doc = doc! {
        "$set" : {
            "name": &updated_event.name,
        "location": &updated_event.location,
        "date": &updated_event.date,
        }
    };
    let filter = doc! {"_id": event_id};
    
    match collection
    .find_one_and_update(filter, update_doc).await {
        Ok(Some(_)) => Ok(Json("User successfully  updated".to_string())),
        Ok(None)=> {
            
            Err(Status::NotFound)
        },
        Err(e)=> {
            Err(Status::InternalServerError)
        }
    }
    
}

#[delete("/event/<event_id>")]
pub async fn drop_event(event_id: &str, db: &State<Collection<Event>>) -> Result<Json<String>, Status> {
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