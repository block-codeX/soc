#[macro_use]
extern crate rocket;


use models::user;
use rocket::{custom, route, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
mod db;
mod routes;
mod models;




#[launch]
async fn rocket() -> _ {
    use std::env;
    let user_db = db::connect::<models::User>().await;
    let event_db = db::connect::<models::Event>().await;
    let application_db = db::connect::<models::Application>().await;
    let blacklisted_tokens_db = db::connect::<models::BlackListedToken>().await;

    let port = env::var("PORT")
    .unwrap_or_else(|_| "8000".to_string() )
    .parse::<u16>()
    .expect("Invalid PORT number");
    
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .unwrap();
    

    rocket::custom(
        rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port,
            ..rocket::Config::default()
        }
    )
    .attach(cors) // Attach CORS Middleware
    .manage(user_db)
    .manage(event_db)
    .manage(application_db)
    .manage(blacklisted_tokens_db)
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
            routes::update_user_rank,
            routes::login,
            routes::profile,
            routes::join_event,
            routes::leave_event,
            routes::get_multiple_events,
            routes::update_pinned,
            routes::delete_all_events,
            routes::delete_all_users,
            routes::read_upcoming_events,
        ],
    )
}
