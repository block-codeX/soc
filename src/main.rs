#[macro_use]
extern crate rocket;

use rocket::routes;
mod db;
mod models;
mod routes;

#[launch]
async fn rocket() -> _ {
    let db = db::connect().await;

    rocket::build().manage(db).mount(
        "/api/v1",
        routes![
            routes::sign_up,
            routes::read_users,
            routes::drop_user,
            routes::update_user,
            routes::read_user
        ],
    )
}
