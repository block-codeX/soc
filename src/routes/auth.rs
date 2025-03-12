use crate::db::Db;
use crate::models::User;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson};
use mongodb::options::FindOptions;
use mongodb::Cursor;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{post, State};

#[post("/user", format = "json", data = "<user>")]
pub async fn sign_up(user: Json<User>, db: &State<Db>) -> Json<String> {
    let new_user = User {
        id: None,
        name: user.name.clone(),
        email: user.email.clone(),
        password: user.password.clone(),
        wallet: user.wallet.clone(),
        user_rank: user.user_rank.or(Some(0)),
    };
    println!("Inserting user: {:?}", new_user);
    let result = db.insert_one(new_user, None).await;

    match result {
        Ok(_) => Json("User registered successfully!".to_string()),
        Err(e) => Json(format!("Error: {e}")),
    }
}

#[get("/users")]
pub async fn read_user(db: &State<Db>) -> Json<Vec<User>> {
    println!("hello");
    let mut cursor: Cursor<User> = db
        .find(None, FindOptions::default())
        .await
        .expect("Failed to find user");
    let mut users: Vec<User> = Vec::new();
    while let Some(user) = cursor.try_next().await.expect("Error iterating cursor") {
        users.push(user);
    }
    Json(users)
}

#[delete("/user/<id>")]
pub async fn drop_user(id: &str, db: &State<Db>) -> Result<Json<String>, Status> {
    let collection = db;

    let object_id = match ObjectId::parse_str(id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest),
    };
    let filter = doc! {"_id": object_id};
    let result = collection.delete_one(filter, None).await;

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
    id: &str,
    updated_user: Json<User>,
    db: &State<Db>,
) -> Result<Json<String>, Status> {
    let collection = db;
    let object_id = match ObjectId::parse_str(id) {
        Ok(oid) => oid,
        Err(_) => return Err(Status::BadRequest),
    };

    let update_doc = doc! {
        "$set": {
            "name": &updated_user.name,
            "email": &updated_user.email,
            "wallet": &updated_user.wallet,
            "user_rank": updated_user.user_rank.map(|rank| Bson::Int32(rank as i32))
        }
    };
    let filter = doc! {"_id": object_id};

    match collection
        .find_one_and_update(filter, update_doc, None)
        .await
    {
        Ok(Some(update_user)) => Ok(Json("User succesfully updated".to_string())),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}
