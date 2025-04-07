use std::time::SystemTime;

use chrono::{format, DateTime, Utc};
use mongodb::Collection;
use rocket::{response::status, serde::json::Json, State};
use crate::{models::{user::{self, UserType}, BlackListedToken, User}, routes::auth::AuthenticatedUser};
use mongodb::bson::{doc, Bson, Uuid, DateTime as BsonDateTime};
use mongodb::Cursor;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use rocket::http::{HeaderMap, Status};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};

use super::auth::{validate_token, AdminUser, AuthToken};


#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub tel: String,
    pub wallet: String,
    pub admin: Option<bool>,
    pub user_type: UserType,
    pub role: String,
    pub stack: Vec<String>,
    pub graduate: bool,
    pub level: i32,
    pub department: String,
    pub university: String,
    pub student: String,

    pub attending_events: Vec<ObjectId>,
    #[serde(with = "chrono::serde::ts_seconds", default = "default_datetime")] // Serialize & Deserialize timestamps properly
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds", default = "default_datetime")]
    pub updated_at: DateTime<Utc>,
}

fn default_datetime() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
pub struct SignUPDto {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub tel: String,
    pub password: String,
    pub wallet: String,
    pub admin: Option<bool>,
    pub user_type: UserType,
    pub role: String,
    pub stack: Vec<String>,
    pub graduate: bool,
    pub level: i32,
    pub department: String,
    pub university: String,
    pub student: String,

    pub attending_events: Vec<ObjectId>,
    #[serde(with = "chrono::serde::ts_seconds", default = "default_datetime")] // Serialize & Deserialize timestamps properly
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds", default = "default_datetime")]
    pub updated_at: DateTime<Utc>,
}



#[get("/profile")]
pub async fn profile(user: AuthenticatedUser, db: &State<Collection<User>>) -> Result<Json<Profile>, status::Custom<String>> {
    let result = db.find_one(doc! {"email": &user.email}).await;

    match result {
        Ok(Some(user_data)) => {
            let return_user = Profile {
                id: user_data.id,
                name: user_data.name,
                email: user_data.email,
                wallet: user_data.wallet,
                admin: user_data.admin,
                attending_events: user_data.attending_events,
                created_at: user_data.created_at,
                updated_at: user_data.updated_at,
                tel: user_data.tel,
                role: user_data.role,
                stack: user_data.stack,
                graduate: user_data.graduate,
                level: user_data.level,
                department: user_data.department,
                university: user_data.university,
                student: user_data.student,
                user_type: user_data.user_type,
            };
            return Ok(Json(return_user));
        },
        Ok(None) => Err(status::Custom(Status::NotFound, "User not found".to_string())), // if no user found
        Err(e) => Err(status::Custom(Status::InternalServerError, format!("Database error: {}", e)))
    }
}

#[post("/user", format = "json", data = "<user>")]
pub async fn sign_up(mut user: Json<SignUPDto>, db: &State<Collection<User>>) -> Result<Json<String>, rocket::response::status::Custom<String>> {
    // Check if the email already exists
    let filter = doc! {"email": &user.email};

    if let Ok(Some(_)) = db.find_one(filter.clone()).await {
        return Err(rocket::response::status::Custom(Status::Conflict, "User already exists".to_string()));
    }

    // Validate user based on type
    match user.user_type {
        UserType::CORETEAM => {
            // Core Team Validation
            user.graduate = true;
            if user.level != 0 || !user.department.is_empty() || !user.university.is_empty() {
                return Err(rocket::response::status::Custom(Status::BadRequest, "Core Team members should not have level, department, or university.".to_string()));
            }
            if !user.stack.is_empty() {
                return Err(rocket::response::status::Custom(Status::BadRequest, "Core Team members should not have a stack.".to_string()));
            }
        }
        UserType::HACKER => {
            // Hacker Validation
            let valid_roles = vec!["backend", "frontend", "smartcontract dev", "product manager", "UIUX"];
            if !valid_roles.contains(&user.role.as_str()) {
                return Err(rocket::response::status::Custom(Status::BadRequest, "Invalid role for hacker.".to_string()));
            }
            if user.graduate && (user.level != 0 || !user.department.is_empty() || !user.university.is_empty()) {
                return Err(rocket::response::status::Custom(Status::BadRequest, "Graduated hackers should not have level, department, or university.".to_string()));
            }
            if !user.graduate && (user.level == 0 || user.department.is_empty() || user.university.is_empty()) {
                return Err(rocket::response::status::Custom(Status::BadRequest, "Non-graduated hackers must have level, department, and university.".to_string()));
            }
        }
        UserType::RANDOM => {
            // Random User Validation
            if !user.role.is_empty() || !user.stack.is_empty() {
                return Err(rocket::response::status::Custom(Status::BadRequest, "Random users should not have role or stack.".to_string()));
            }
            if user.graduate && (user.level != 0 || !user.department.is_empty() || !user.university.is_empty()) {
                return Err(rocket::response::status::Custom(Status::BadRequest, "Graduated random users should not have level, department, or university.".to_string()));
            }
        }
    }

    // Hash the password
    let hashed_password = match hash(&user.password, DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(_) => return Err(rocket::response::status::Custom(Status::Conflict, "Error hashing password".to_string())),
    };

    let now = Utc::now();

    // Create new user
    let new_user = User {
        id: None,
        name: user.name.clone(),
        email: user.email.clone(),
        tel: user.tel.clone(),
        password: hashed_password,
        wallet: user.wallet.clone(),
        admin: Some(false),
        user_type: user.user_type.clone(),
        role: user.role.clone(),
        stack: user.stack.clone(),
        graduate: user.graduate,
        level: user.level,
        department: user.department.clone(),
        university: user.university.clone(),
        student: user.student.clone(),
        attending_events: vec![],
        created_at: now,
        updated_at: now,
    };

    // Insert the new user into the database
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
    if let Some(rank) = updated_user.admin {
        update_doc.insert("user_rank", Bson::Boolean(rank));
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


#[put("/user/<id>/admin", format = "json", data = "<admin>")]
pub async fn update_user_rank(
    id: &str,
    admin: Json<bool>,
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
    let update = doc! {"$set": {"admin": Bson::Boolean(*admin)}};

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

#[delete("/users")]
pub async fn delete_all_users(
    database: &State<Collection<User>>,  // Assuming you're using a `User` collection
    _admin: AdminUser, // Only admin can call this
    _token: AuthToken,  // Verify blacklisted tokens
    _user: AuthenticatedUser, // Verify authenticated user
) -> Json<String> {
    // Delete all users in the collection
    let result = database.delete_many(doc! {}).await;

    match result {
        Ok(delete_result) => {
            if delete_result.deleted_count > 0 {
                Json("All users successfully deleted.".to_string())
            } else {
                Json("No users were found to delete.".to_string())
            }
        }
        Err(_) => Json("Failed to delete users.".to_string()),
    }
}
