use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{
    delete, get, http, patch, post, web, App, Either, HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use indoc::indoc;
use log::info;
use serde::{Deserialize, Serialize};

use crate::state::State;

mod state;

type Id = u32;

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
) -> Either<HttpResponse, web::Json<Task>> {
    let path = path.into_inner();

    match sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(path.id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(task) => match task {
            Some(task) => Either::Right(web::Json(task)),
            None => Either::Left(HttpResponse::NoContent().finish()),
        },

        Err(e) => {
            info!("{e:?}");
            Either::Left(HttpResponse::BadRequest().body(format!("{:?}", e)))
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
    use actix_web::test;
    use sqlx::Executor;

    use super::*;

    #[actix_web::test]
    async fn test_get_tasks() -> anyhow::Result<()> {
        let app = App::new()
            .service(get_tasks)
            .app_data(web::Data::new(State::init_and_migrate().await?));
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
        let app = App::new()
            .app_data(web::Data::new(State::init_and_migrate().await?))
            .service(get_task);
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/tasks/1").to_request();
        let resp: Task = test::call_and_read_body_json(&app, req).await;

        let task = Task {
            id: 1,
            title: "First Task".into(),
            done: false,
        };
        assert_eq!(resp, task);

        // id is not found
        let req = test::TestRequest::get().uri("/tasks/99999").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        Ok(())
    }

    #[actix_web::test]
    async fn test_post_tasks() -> anyhow::Result<()> {
        let state = State::init().await?;

        state
            .db
            .execute(indoc!(
                "
                CREATE TABLE IF NOT EXISTS tasks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title STRING NOT NULL,
                    done INTEGER NOT NULL DEFAULT 0
            );"
            ))
            .await?;

        let app = App::new()
            .app_data(web::Data::new(state))
            .service(post_tasks)
            .service(get_tasks);
        let app = test::init_service(app).await;

        // post
        {
            let task = AddTask {
                title: "Test Task Test".into(),
            };

            let req = test::TestRequest::post()
                .uri("/tasks")
                .set_json(&task)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), http::StatusCode::OK);
        }

        // get
        {
            let req = test::TestRequest::get().uri("/tasks").to_request();
            let resp: Vec<Task> = test::call_and_read_body_json(&app, req).await;

            assert_eq!(
                resp,
                vec! {Task {
                    id: 1,
                    title: "Test Task Test".into(),
                    done: false,
                }}
            );
        }
        Ok(())
    }

    #[actix_web::test]
    async fn test_delete_task() -> anyhow::Result<()> {
        let state = State::init_and_migrate().await?;

        let app = App::new()
            .app_data(web::Data::new(state))
            .service(delete_task)
            .service(get_task);
        let app = test::init_service(app).await;

        let path = "/tasks/1";

        // fount task
        {
            let req = test::TestRequest::get().uri(path).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), http::StatusCode::OK);
        }

        // delete task
        {
            let req = test::TestRequest::delete().uri(path).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), http::StatusCode::OK);
        }

        // not found task
        {
            let req = test::TestRequest::get().uri(path).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);
        }

        Ok(())
    }
}
