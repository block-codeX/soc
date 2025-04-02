use std::time::SystemTime;

use chrono::{format, Utc};
use mongodb::Collection;
use rocket::{serde::json::Json, State};
use serde_json::Error;
use crate::{models::{user, BlackListedToken, User}, routes::auth::AuthenticatedUser};
use mongodb::bson::{doc, Bson, Uuid, DateTime as BsonDateTime};
use mongodb::Cursor;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use rocket::http::{HeaderMap, Status};
use bcrypt::{hash, DEFAULT_COST};

use super::auth::{validate_token, AuthToken};

#[get("/profile")]
pub fn profile(user: AuthenticatedUser) -> Json<String> {
    Json(format!("welcome, {}", user.email))
}

#[post("/user", format = "json", data = "<user>")]
pub async fn sign_up(user: Json<User>, db: &State<Collection<User>>) -> Result< Json<String> ,rocket::response::status::Custom<String>> {
    
    let filter = doc! {"email": &user.email};

    if let Ok(Some(_)) = db.find_one(filter.clone()).await {
        return Err(rocket::response::status::Custom(Status::Conflict, "User already exist".to_string()));
    }
    
    let hashed_password = match hash(&user.password, DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(_) => return Err(rocket::response::status::Custom(Status::Conflict, "Error hashing password".to_string())),
    };
    let now= Utc::now();
    let new_user = User {id:None,name:user.name.clone(),
        email:user.email.clone(),
        password:hashed_password,
        wallet:user.wallet.clone(),
        user_rank:user.user_rank.or(Some(0)), 
        created_at: now, 
        updated_at: now,
    };
    
    // println!("Inserting user: {:?}", new_user);
    let result = db.insert_one(new_user).await;


    match result {
        Ok(_) => Ok(Json("User registered successfully!".to_string())),
        Err(e) => Err(rocket::response::status::Custom(Status::BadRequest, format!("Error: {e}"))),
    }
}

#[get("/users")]
pub async fn read_users(db: &State<Collection<User>>,
     _user: AuthenticatedUser,
     _token: AuthToken,
    //  db_blacklist: &State<Collection<BlackListedToken>>
    ) -> Json<Vec<User>> {
    let mut cursor: Cursor<User> = db
        .find( doc! {})
        .await
        .expect("Failed to find user");
    let mut users: Vec<User> = Vec::new();
    while let Some(user) = cursor.try_next().await.expect("Error iterating cursor") {
        users.push(user);
    }
    Json(users)
}

#[get("/user/<id>")]
pub async  fn read_user(db: &State<Collection<User>>,
     id: &str, 
     _token: AuthToken,
     _user: AuthenticatedUser) -> Result<Json<User>, Status> {
    let collection = db;
    let object_id = match ObjectId::parse_str(id) {
        Ok(oid)=> oid,
        Err(_) => return Err(Status::BadRequest),
    };
    let filter = doc! {"_id": object_id};
    let result = collection.find_one(filter
        
    ).await;
    match result {
        Ok(fetched_data) => {
            if let Some(data) = fetched_data {
                Ok(Json(data))
            }else {
                Err(Status::NotFound)
            }
        }
        Err(_) => Err(Status::InternalServerError)
    }
}

#[delete("/user/<id>")]
pub async fn drop_user(id: &str, 
    db: &State<Collection<User>>,
    _token: AuthToken,
     _user: AuthenticatedUser) -> Result<Json<String>, Status> {
    let collection = db;

    let object_id = match ObjectId::parse_str(id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest),
    };
    let filter = doc! {"_id": object_id};
    let result = collection.delete_one(filter).await;

    match result {
        Ok(delete_result) => {
            if delete_result.deleted_count > 0 {
                Ok(Json("User deleted successfully!".to_string()))
            } else {
                Err(Status::NotFound)
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}


#[put("/user/<id>", format = "json", data = "<updated_user>")]
pub async fn update_user(
    _user: AuthenticatedUser,
    id: &str,
    updated_user: Json<User>,
    _token: AuthToken,
    db: &State<Collection<User>>,
) -> Result<Json<String>, Status> {
    let collection = db;
    let object_id = match ObjectId::parse_str(id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest),
    };

    let mut update_doc = doc! {
        
            "name": &updated_user.name,
            "email": &updated_user.email,
            "wallet": &updated_user.wallet,
            "updated_at": BsonDateTime::from(SystemTime::from(Utc::now())),
    
    };
    // only add user ranking if it is provided
    if let Some(rank) = updated_user.user_rank {
        update_doc.insert("user_rank", Bson::Int32(rank));
    }
    
    let updated_doc = doc! {  "$set": Bson::Document(update_doc)};
    
    let filter = doc! {"_id": object_id};

    match collection
        .find_one_and_update(filter, updated_doc)
        .await
    {
        Ok(Some(_)) => Ok(Json("User succesfully updated".to_string())),
        Ok(None) => {
            eprintln!("User not found: {}", id);
            Err(Status::NotFound)
        },
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(Status::InternalServerError)},
    }
}


#[put("/user/<id>/rank", format = "json", data = "<new_rank>")]
pub async fn update_user_rank(
    id: &str,
    new_rank: Json<i32>,
    db: &State<Collection<User>>,
    _token: AuthToken,
    _user: AuthenticatedUser,
) -> Result<Json<String>, Status> {
    let collection = db;

    let object_id = match ObjectId::parse_str(id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest)
    };

    let filter = doc! {"_id": object_id};
    let update = doc! {"$set": {"user_rank": Bson::Int32(*new_rank)}};

    match collection.find_one_and_update(filter, update).await {
        Ok(Some(_)) => Ok(Json("User rank successfully updated".to_string())),
        Ok(None) => {
            eprintln!("User not found: {}", id);
            Err(Status::NotFound)
        },
        Err(e) => {
            eprintln!("Database error : {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}


