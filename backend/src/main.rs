use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{delete, get, http, post, put, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

type Id = u32;

#[derive(Debug, Clone)]
struct State {
    db: SqlitePool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Task {
    id: Id,
    title: String,
    done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskPath {
    id: Id,
}

#[get("/tasks/{id}")]
async fn get_task(
    state: web::Data<State>,
    path: web::Path<TaskPath>,
) -> actix_web::Result<web::Json<Task>> {
    let path = path.into_inner();

    match sqlx::query_as::<_, Task>("SELECT * FROM task WHERE id = ?")
        .bind(path.id)
        .fetch_one(&state.db)
        .await
    {
        Ok(task) => Ok(web::Json(task)),
        Err(e) => {
            info!("{e:?}");
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

#[get("/tasks")]
async fn get_tasks(state: web::Data<State>) -> actix_web::Result<web::Json<Vec<Task>>> {
    match sqlx::query_as::<_, Task>("SELECT * FROM tasks")
        .fetch_all(&state.db)
        .await
    {
        Ok(tasks) => Ok(web::Json(tasks)),
        Err(e) => {
            info!("{e:?}");
            Err(actix_web::error::ErrorBadRequest(e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AddTask {
    title: String,
}

#[post("/task")]
async fn post_task(state: web::Data<State>, json: web::Json<AddTask>) -> impl Responder {
    if let Err(e) = sqlx::query("INSERT INTO tasks (title) VALUES (?)")
        .bind(json.title.clone())
        .execute(&state.db)
        .await
    {
        info!("{:?}", e);
        HttpResponse::BadRequest()
    } else {
        HttpResponse::Ok()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteTask {
    id: Id,
}

#[delete("/tasks")]
async fn delete_task(state: web::Data<State>, json: web::Json<DeleteTask>) -> impl Responder {
    if let Err(e) = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(json.id)
        .execute(&state.db)
        .await
    {
        info!("{:?}", e);
        HttpResponse::BadRequest()
    } else {
        HttpResponse::Ok()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PutTask {
    title: Option<String>,
    done: Option<bool>,
}

#[put("/tasks/{id}")]
async fn put_task(
    state: web::Data<State>,
    path: web::Path<TaskPath>,
    json: web::Json<PutTask>,
) -> impl Responder {
    let path = path.into_inner();

    let pre = "UPDATE tasks SET";
    let post = "WHERE id = ?";
    match (&json.title, &json.done) {
        (Some(title), Some(done)) => {
            if let Err(e) = sqlx::query(&format!("{pre} title = ? done = ? {post}"))
                .bind(title)
                .bind(if *done { 1 } else { 0 })
                .bind(path.id)
                .execute(&state.db)
                .await
            {
                info!("{e:?}");
                HttpResponse::BadRequest()
            } else {
                HttpResponse::Ok()
            }
        }
        (Some(title), None) => {
            if let Err(e) = sqlx::query(&format!("{pre} title = ? {post}"))
                .bind(title)
                .bind(path.id)
                .execute(&state.db)
                .await
            {
                info!("{e:?}");
                HttpResponse::BadRequest()
            } else {
                HttpResponse::Ok()
            }
        }
        (None, Some(done)) => {
            dbg!(done);
            if let Err(e) = sqlx::query(&format!("{pre} done = ? {post}"))
                .bind(if *done { 1 } else { 0 })
                .bind(path.id)
                .execute(&state.db)
                .await
            {
                info!("{e:?}");
                HttpResponse::BadRequest()
            } else {
                HttpResponse::Ok()
            }
        }
        (None, None) => HttpResponse::Ok(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let state = State {
        db: SqlitePoolOptions::new()
            .max_connections(10)
            .connect(":memory:")
            .await?,
    };

    sqlx::migrate!().run(&state.db).await?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req| {
                let host = origin.to_str().unwrap();
                host.contains("127.0.0.1") || host.contains("localhost")
            })
            .allowed_methods(vec!["HEAD", "GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![http::header::ACCEPT, http::header::CONTENT_TYPE])
            .max_age(3600);
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .route("/", web::get().to(HttpResponse::Ok))
            .service(get_tasks)
            .service(post_task)
            .service(delete_task)
            .service(put_task)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
