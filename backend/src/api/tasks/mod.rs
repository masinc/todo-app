use actix_web::web;
use serde::{Deserialize, Serialize};

pub mod delete_tasks;
pub mod get_task;
pub mod get_tasks;
pub mod patch_tasks;
pub mod post_tasks;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskPath {
    pub id: crate::model::Id,
}

pub fn init(cfg: &mut web::ServiceConfig) -> anyhow::Result<()> {
    cfg.service(get_tasks::get_tasks);
    cfg.service(get_task::get_task);
    cfg.service(post_tasks::post_tasks);
    cfg.service(delete_tasks::delete_tasks);
    cfg.service(patch_tasks::patch_task);
    Ok(())
}
