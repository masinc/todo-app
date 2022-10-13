use actix_web::web;

pub mod tasks;

pub fn init(cfg: &mut web::ServiceConfig) -> anyhow::Result<()> {
    tasks::init(cfg)?;
    Ok(())
}
