use actix_web::{delete, web, HttpResponse, Responder};
use log::info;

use crate::state::State;

use super::TaskPath;

#[delete("/tasks/{id}")]
pub async fn delete_tasks(state: web::Data<State>, path: web::Path<TaskPath>) -> impl Responder {
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

#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};

    use super::*;

    //noinspection DuplicatedCode
    //noinspection DuplicatedCode
    #[actix_web::test]
    async fn test_delete_task() -> anyhow::Result<()> {
        let state = State::init_and_migrate().await?;

        let app = App::new()
            .app_data(web::Data::new(state))
            .configure(|cfg| crate::api::tasks::init(cfg).unwrap());

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
