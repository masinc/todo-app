use actix_web::{patch, web, HttpResponse, Responder};
use log::info;
use serde::{Deserialize, Serialize};

use crate::api::tasks::TaskPath;
use crate::state::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchTask {
    pub title: Option<String>,
    pub done: Option<bool>,
}

#[patch("/tasks/{id}")]
pub async fn patch_task(
    state: web::Data<State>,
    path: web::Path<TaskPath>,
    json: web::Json<PatchTask>,
) -> impl Responder {
    let path = path.into_inner();

    let pre = "UPDATE tasks SET";
    let post = "WHERE id = ?";
    match (&json.title, &json.done) {
        (Some(title), Some(done)) => {
            if let Err(e) = sqlx::query(&format!("{pre} title = ?, done = ? {post}"))
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

#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};

    use crate::model::Task;

    use super::*;

    //noinspection DuplicatedCode
    #[actix_web::test]
    async fn test_patch_task() -> anyhow::Result<()> {
        let state = State::init_and_migrate().await?;

        let app = App::new()
            .app_data(web::Data::new(state))
            .configure(|cfg| crate::api::tasks::init(cfg).unwrap());
        let app = test::init_service(app).await;

        let path = "/tasks/1";

        // found task
        {
            let req = test::TestRequest::get().uri(path).to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), http::StatusCode::OK);
        }

        // patch task
        {
            let req = test::TestRequest::patch()
                .uri(path)
                .set_json(PatchTask {
                    title: Some("test_patch_task".into()),
                    done: Some(true),
                })
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), http::StatusCode::OK);
        }

        // get patched task
        {
            let req = test::TestRequest::get().uri(path).to_request();
            let resp: Task = test::call_and_read_body_json(&app, req).await;

            assert_eq!(resp.title, "test_patch_task".to_string());
            assert!(resp.done);
        }

        Ok(())
    }
}
