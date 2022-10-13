use actix_web::{get, web};
use log::info;

use crate::model::Task;
use crate::state::State;

#[get("/tasks")]
pub async fn get_tasks(state: web::Data<State>) -> actix_web::Result<web::Json<Vec<Task>>> {
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

#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    use super::*;

    //noinspection DuplicatedCode
    #[actix_web::test]
    async fn test_get_tasks() -> anyhow::Result<()> {
        let app = App::new()
            .app_data(web::Data::new(State::init_and_migrate().await?))
            .configure(|cfg| crate::api::tasks::init(cfg).unwrap());
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
}
