use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::opt::RecordId;

use crate::task::Priority;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: RecordId,
    pub title: String,
    pub priority: Priority,
    pub due: Option<DateTime<Utc>>,
    pub sub: Vec<Task>,
}

//impl From<crate::task::Task> for Task {
//    fn from(value: crate::task::Task) -> Self {
//        Self {
//            id: RecordId::from_str("").unwrap(),
//            title: value.title,
//            priority: value.priority,
//            due: value.due,
//            sub: value
//                .sub
//                .into_iter()
//                .map(|subtask| subtask.into())
//                .collect(),
//        }
//    }
//}
