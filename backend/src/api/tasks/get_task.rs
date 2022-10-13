use actix_web::{get, web, Either, HttpResponse};
use log::info;

use crate::api::tasks::TaskPath;
use crate::model::Task;
use crate::state::State;

#[get("/tasks/{id}")]
pub async fn get_task(
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

#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};

    use super::*;

    //noinspection DuplicatedCode
    #[actix_web::test]
    async fn test_get_task() -> anyhow::Result<()> {
        let app = App::new()
            .app_data(web::Data::new(State::init_and_migrate().await?))
            .configure(|cfg| crate::api::tasks::init(cfg).unwrap());
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
}
