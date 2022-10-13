use actix_web::{post, web, HttpResponse, Responder};
use log::info;
use serde::{Deserialize, Serialize};

use crate::state::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostTask {
    pub title: String,
}

#[post("/tasks")]
pub async fn post_tasks(state: web::Data<State>, json: web::Json<PostTask>) -> impl Responder {
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

#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};
    use indoc::indoc;
    use sqlx::Executor;

    use crate::model::Task;

    use super::*;

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
            .configure(|cfg| crate::api::tasks::init(cfg).unwrap());
        let app = test::init_service(app).await;

        // post
        {
            let task = PostTask {
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
}
