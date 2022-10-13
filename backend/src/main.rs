use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http, web, App, HttpResponse, HttpServer};
use env_logger::Env;

use crate::state::State;

mod api;
mod model;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let state = State::init_and_migrate().await?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req| {
                let host = origin.to_str().unwrap();
                host.contains("127.0.0.1") || host.contains("localhost")
            })
            .allowed_methods(vec![
                "HEAD", "OPTIONS", "GET", "POST", "PUT", "DELETE", "PATCH",
            ])
            .allowed_headers(vec![http::header::ACCEPT, http::header::CONTENT_TYPE])
            .max_age(3600);
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .route("/", web::get().to(HttpResponse::Ok))
            .configure(|cfg| api::init(cfg).unwrap())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
