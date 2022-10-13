use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, PartialEq, Eq, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: crate::model::Id,
    pub title: String,
    pub done: bool,
}
