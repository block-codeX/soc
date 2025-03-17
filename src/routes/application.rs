use futures::TryStreamExt;
use mongodb::Cursor;
use rocket::{post, http::Status, State};
use rocket::serde::json::Json;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Collection;
use serde::{Serialize, Deserialize};
use crate::models::{ Application, ApplicationStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyRequest {
    pub user_id: String,
    pub event_id: String,
}

#[post("/apply", format="json", data="<apply_req>")]
pub async fn apply_for_event(
    db: &State<Collection<Application>>,
    apply_req: Json<ApplyRequest>
) -> Result<Status, Status> {
    let user_id = ObjectId::parse_str(&apply_req.user_id).map_err(|_| Status::BadRequest)?;
    let event_id = ObjectId::parse_str(&apply_req.event_id).map_err(|_| Status::BadRequest)?;

    let existing = db
    .find_one(doc! 
        {
            "user_id": &user_id,
            "event_id" : &event_id
        }
    ).await
    .map_err(|_| Status::InternalServerError)?;

    if existing.is_some() {
        return Err(Status::Conflict);
    };

    let application = Application {
        id: ObjectId::new(),
        user_id,
        event_id,
        status: ApplicationStatus::Pending
    };

    db.insert_one(application)
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Status::Created)
}

#[get("/applicants")]
pub async  fn read_applicants(db: &State<Collection<Application>>) -> Json<Vec<Application>> {
    let mut cursor: Cursor<Application> = db.find(doc! {})
    .await
    .expect("failed to find user");
    let mut applications = Vec::new();
    while let Some(application) = cursor.try_next().await.expect("Error interating cursor") {
        applications.push(application);

    }
    Json(applications)
}