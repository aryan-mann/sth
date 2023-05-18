use actix_web::{web, Responder, HttpResponse};
use sqlx::{PgPool};

use crate::models::Task;
use crate::db::DbOps;

// TODO: Move these to another file
mod requests {
    use serde::{Serialize, Deserialize};
    use chrono::{DateTime,Utc};
    use crate::models::TaskType;

    #[derive(Serialize, Deserialize)]
    pub struct CreateTaskRequest {
        pub task_type: TaskType,
        pub execution_time: DateTime<Utc>,
        pub repeat: bool
    }

    #[derive(Serialize, Deserialize)]
    pub struct GetTasksRequest {
        pub task_type: Option<TaskType>
    }

}

// TODO: Move these to a struct with functions rather than just free floating
/* Create a Task */
pub async fn create_task(db: web::Data<PgPool>, info: web::Json<requests::CreateTaskRequest>) -> impl Responder {
    let new_task = Task {
        id: 0,
        task_type: info.task_type,
        scheduled_for: info.execution_time,
        repeat: info.repeat,
        last_run: None
    };

    let db = db.into_inner();
    match DbOps::create_task(db.clone(), new_task).await {
        Ok(new_task_id) => {
            let resp_text = format!("New task created with id '{}'", new_task_id);
            HttpResponse::Ok().body(resp_text)
        },
        Err(err) => HttpResponse::InternalServerError().body(Into::<String>::into(err)),
    }
}

/* Get all Task's */
pub async fn get_all_tasks(db: web::Data<PgPool>) -> impl Responder {
    let db = db.into_inner();
    match DbOps::get_all_tasks(db).await {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(err) => HttpResponse::InternalServerError().body(Into::<String>::into(err)),
    }
}

/* Get particular Task */
pub async fn show_task(db: web::Data<PgPool>, path: web::Path<i32>) -> impl Responder {
    let task_id = path.into_inner();
    let db = db.into_inner();

    match DbOps::get_task(db, task_id).await {
        Ok(task) => HttpResponse::Ok().json(task),
        Err(err) => HttpResponse::BadRequest().body(Into::<String>::into(err))
    }
}

/* Delete a Task */
pub async fn delete_task(db: web::Data<PgPool>, path: web::Path<i32>) -> impl Responder {
    let task_id = path.into_inner();
    let db = db.into_inner();

    match DbOps::delete_task(db, task_id).await {
        Ok(affected_rows) => {
            let resp_text = format!("{} row(s) deleted", affected_rows);
            HttpResponse::Ok().body(resp_text)
        },
        Err(err) => HttpResponse::BadRequest().body(Into::<String>::into(err))
    }
}
