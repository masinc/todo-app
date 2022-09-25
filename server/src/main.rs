use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
struct State {
    db: SqlitePool,
}

#[get("/version")]
async fn version(state: web::Data<State>) -> impl Responder {
    let v: (String,) = sqlx::query_as("SELECT sqlite_version()")
        .fetch_one(&state.db)
        .await
        .unwrap();

    v.0
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State {
        db: SqlitePoolOptions::new()
            .max_connections(10)
            .connect(":memory:")
            .await?,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/", web::get().to(HttpResponse::Ok))
            .service(version)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
