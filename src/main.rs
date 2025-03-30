#[macro_use]
extern crate rocket;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use models::user;
use rocket::{custom, routes};
mod db;
mod routes;
mod models;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, Ninja!")
}


#[launch]
async fn rocket() -> _ {
    use std::env;
    let user_db = db::connect::<models::User>().await;
    let event_db = db::connect::<models::Event>().await;
    let application_db = db::connect::<models::Application>().await;

    let port = env::var("PORT")
    .unwrap_or_else(|_| "8000".to_string() )
    .parse::<u16>()
    .expect("Invalid PORT number");

    

    rocket::custom(
        rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port,
            ..rocket::Config::default()
        }
    )
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
            routes::update_user_rank
        ],
    )
}
