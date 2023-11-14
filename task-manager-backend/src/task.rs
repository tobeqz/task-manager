use std::{collections::HashMap, marker::PhantomData};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal, opt::PatchOp};

use crate::{db_types, ServerState};

#[derive(Serialize, Deserialize, Debug)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub title: String,
    pub priority: Priority,
    pub due: Option<DateTime<Utc>>,
    pub sub: Vec<Subtask>,
    pub done: bool,
    pub done_at: Option<DateTime<Utc>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subtask {
    pub title: String,
    pub done: bool,
    pub sub: Vec<Subtask>
}

#[derive(Deserialize, Debug)]
pub struct CreateTask {
    title: String,
    priority: Priority,
    due: Option<DateTime<Utc>>,
}

impl Into<Task> for CreateTask {
    fn into(self) -> Task {
        Task {
            title: self.title,
            priority: self.priority,
            due: self.due,
            sub: vec![],
            done: false,
            done_at: None
        }
    }
}

pub async fn get_tasks(
    State(ServerState { db, .. }): State<ServerState>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let mut result = match db.query("SELECT * FROM task").await {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    };

    let tasks: Vec<Task> = match result.take(0) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        },
    };

    Ok(Json::from(tasks))
}

pub async fn create_task(
    State(ServerState { db, .. }): State<ServerState>,
    Json(task_info): Json<CreateTask>,
) -> Result<Json<String>, StatusCode> {
    let task: Task = task_info.into();

    let created: Vec<db_types::Task> = match db.create("task").content(task).await {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let created = match created.get(0) {
        Some(x) => x,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Enclose the string in quotes, because the `.into()` does not actually serialize the string
    // as JSON, it just sends the string directly and assumes it is already serialized
    let id = format!("\"{}\"", created.id.id.to_string());
    Ok(id.into())
}

pub async fn delete_task(
    State(ServerState { db, .. }): State<ServerState>,
    Path(params): Path<HashMap<String, String>>,
) -> Result<(), StatusCode> {
    let id = match params.get("id") {
        Some(id) => id,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let deleted: Option<Task> = match db.delete(("task", id)).await {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e); // This does not fail when it can't find the id, something else
                                // must have gone wrong.
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let None = deleted {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(())
}

pub async fn update_title(
    State(ServerState { db, .. }): State<ServerState>,
    Path(params): Path<HashMap<String, String>>,
    Json(new_title): Json<String>,
) -> Result<(), StatusCode> {
    let id = match params.get("id") {
        Some(id) => id,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let patch = PatchOp::replace("title", new_title);

    let updated: Option<Task> = match db
        .update(("task", id))
        .patch(patch)
        .await
    {
        Ok(x) => x,
        Err(..) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if let None = updated {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(())
}

pub async fn update_priority(
    State(ServerState { db, .. }): State<ServerState>,
    Path(params): Path<HashMap<String, String>>,
    Json(new_priority): Json<Priority>,
) -> Result<(), StatusCode> {
    let id = match params.get("id") {
        Some(id) => id,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let patch = PatchOp::replace("priority", new_priority);

    let updated: Option<Task> = match db.update(("task", id)).patch(patch).await {
        Ok(x) => x,
        Err(..) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if let None = updated {
        return Err(StatusCode::NOT_FOUND)
    }

    Ok(())
}

pub async fn update_due_date(
    State(ServerState { db, .. }): State<ServerState>,
    Path(params): Path<HashMap<String, String>>,
    Json(new_date): Json<DateTime<Utc>>,
) -> Result<(), StatusCode> {
    let id = match params.get("id") {
        Some(id) => id,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let patch = PatchOp::replace("due", new_date);

    let updated: Option<Task> = match db.update(("task", id)).patch(patch).await {
        Ok(x) => x,
        Err(..) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };

    if let None = updated {
        return Err(StatusCode::NOT_FOUND)
    }

    Ok(())
}

pub async fn update_done(
    State(ServerState { db, .. }): State<ServerState>,
    Path(params): Path<HashMap<String, String>>,
    Json(new_value): Json<bool>,
) -> Result<(), StatusCode> {
    let id = match params.get("id") {
        Some(id) => id,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let update_query = db.update(("task", id));
    let mut update_query = update_query.patch(PatchOp::replace("done", new_value));
    
    if new_value {
        // If the task is marked as done, overwrite the done_at date
        update_query = update_query.patch(PatchOp::replace("done_at", chrono::Utc::now()));
    }

    let update_query = update_query;

    let updated: Option<Task> = match update_query.await {
        Ok(x) => x,
        Err(..) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if let None = updated {
        return Err(StatusCode::NOT_FOUND)
    }

    todo!()
}

//#[derive(Serialize, Deserialize)]
//struct SubtaskPath(Vec<usize>);
//
//pub struct SubtaskInfo {
//    title: String,
//    path: SubtaskPath
//}
//
//pub async fn create_subtask(
//    State(ServerState { db, .. }): State<ServerState>,
//    Path(params): Path<HashMap<String, String>>,
//    Json(subtask_info): Json<SubtaskInfo>
//) {}
