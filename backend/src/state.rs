#[derive(Debug, Clone)]
pub struct State {
    pub db: sqlx::SqlitePool,
}

impl State {
    pub async fn init() -> anyhow::Result<State> {
        let state = State {
            db: sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(10)
                .connect(":memory:")
                .await?,
        };

        Ok(state)
    }

    pub async fn init_and_migrate() -> anyhow::Result<State> {
        let state = State::init().await?;
        sqlx::migrate!().run(&state.db).await?;

        Ok(state)
    }
}
