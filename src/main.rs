#[macro_use]
extern crate rocket;

use models::user;
use rocket::routes;
mod db;
mod routes;
mod models;

#[launch]
async fn rocket() -> _ {
    let user_db = db::connect::<models::User>().await;
    let event_db = db::connect::<models::Event>().await;
    let application_db = db::connect::<models::Application>().await;

    rocket::build()
    .manage(user_db)
    .manage(event_db)
    .manage(application_db)
    .mount(
        "/api/v1",
        routes![
            routes::sign_up,
            routes::read_users,
            routes::drop_user,
            routes::update_user,
            routes::read_user,
            routes::create_event,
            routes::read_event,
            routes::read_events,
            routes::update_event,
            routes::drop_event,
            routes::apply_for_event,
            routes::read_applicants,
        ],
    )
}
