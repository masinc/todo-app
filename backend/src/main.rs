use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{delete, get, http, patch, post, web, App, HttpResponse, HttpServer, Responder};
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

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize, sqlx::FromRow)]
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

    match sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
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

#[post("/tasks")]
async fn post_tasks(state: web::Data<State>, json: web::Json<AddTask>) -> impl Responder {
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

#[delete("/tasks/{id}")]
async fn delete_task(state: web::Data<State>, path: web::Path<TaskPath>) -> impl Responder {
    let path = path.into_inner();
    if let Err(e) = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(path.id)
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

#[patch("/tasks/{id}")]
async fn patch_task(
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

async fn init_state() -> anyhow::Result<State> {
    let state = State {
        db: SqlitePoolOptions::new()
            .max_connections(10)
            .connect(":memory:")
            .await?,
    };

    sqlx::migrate!().run(&state.db).await?;
    Ok(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let state = init_state().await?;

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
            .service(get_tasks)
            .service(post_tasks)
            .service(delete_task)
            .service(patch_task)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_get_tasks() -> anyhow::Result<()> {
        let app = App::new()
            .service(get_tasks)
            .app_data(web::Data::new(init_state().await?));
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/tasks").to_request();
        let resp: Vec<Task> = test::call_and_read_body_json(&app, req).await;

        let tasks = vec![
            Task {
                id: 1,
                title: "First Task".into(),
                done: false,
            },
            Task {
                id: 2,
                title: "Second Task".into(),
                done: false,
            },
        ];

        assert_eq!(resp, tasks);

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_task() -> anyhow::Result<()> {
        let app = App::new().app_data(web::Data::new(init_state().await?)).service(get_task);
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/tasks/1").to_request();
        let resp: Task = test::call_and_read_body_json(&app, req).await;

        let task = Task {
            id: 1,
            title: "First Task".into(),
            done: false,
        };

        assert_eq!(resp, task);

        Ok(())
    }
}
