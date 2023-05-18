use std::sync::Arc;

use chrono::{Utc, Duration};
use sqlx::{PgPool, Row, Executor};

use crate::{models::Task, common::SvixError};

// Operations on the database
pub struct DbOps;

impl DbOps {
    pub async fn get_all_tasks(db: Arc<PgPool>) -> Result<Vec<Task>, SvixError> {
        let fetch_all_query = sqlx::query("SELECT * FROM Tasks");

        let Ok(rows) = db.fetch_all(fetch_all_query).await else {
                return Err(SvixError::new("Unable to query all tasks"));
            };

        let mut tasks: Vec<Task> = vec![];
        for row in rows {
            match TryInto::<Task>::try_into(row) {
                Ok(task) => tasks.push(task),
                Err(_) => {},
            }
        }
        Ok(tasks)
    }

    pub async fn get_all_tasks_before(db: Arc<PgPool>, max_time: chrono::DateTime<Utc>) -> Result<Vec<Task>, SvixError> {
        let fetch_query = sqlx::query("SELECT * FROM Tasks WHERE scheduled_for < $1")
            .bind(max_time);

        let Ok(rows) = db.fetch_all(fetch_query).await else {
            return Err(SvixError::new("Unable to query all tasks under specified time"));
        };

        let mut tasks: Vec<Task> = vec![];
        for row in rows {
            match TryInto::<Task>::try_into(row) {
                Ok(task) => tasks.push(task),
                Err(_) => {},
            }
        }
        Ok(tasks)
    }

    pub async fn get_task(db: Arc<PgPool>, task_id: i32) -> Result<Task, SvixError> {
        let fetch_query = sqlx::query("SELECT * FROM Tasks where id = $1")
            .bind(task_id);

        let Ok(row) = db.fetch_one(fetch_query).await else {
            return Err(SvixError::new("Unable to fetch task with given id"));
        };

        TryInto::<Task>::try_into(row).or_else(|e| Err(e))
    }

    pub async fn create_task(db: Arc<PgPool>, task: Task) -> Result<i32, SvixError> {
        // Validation

        // nearest date time should be atleast five minutes from now
        let nearest_valid_utc_time = Utc::now() + Duration::minutes(5);
        if task.scheduled_for < nearest_valid_utc_time {
            return Err(SvixError::new("you need to schedule a task atleast 5 minutes from now"));
        }

        // SQL query
        let insertion_query  = sqlx::query("INSERT INTO Tasks (task_type, scheduled_for, repeat) VALUES ($1, $2, $3) RETURNING ID")
            .bind(Into::<String>::into(task.task_type))
            .bind(task.scheduled_for)
            .bind(task.repeat);

        let insertion = db.fetch_one(insertion_query).await;

        match insertion {
            Ok(row) => {
                let Ok(id) = row.try_get::<i32, _>("id") else { return Err(SvixError::new("Could not parse 'id' field of Task")); };
                return Ok(id);
            },
            Err(err) => {
                return Err(SvixError::new(&err.to_string()))
            },
        };
    }

    pub async fn delete_task(db: Arc<PgPool>, task_id: i32) -> Result<u64, SvixError> {
        let delete_query = sqlx::query("DELETE FROM Tasks WHERE id = $1")
            .bind(task_id);

        match db.execute(delete_query).await {
            Ok(res) => Ok(res.rows_affected()),
            Err(err) => Err(SvixError::Unknown { message: err.to_string() })
        }
    }

    pub async fn delete_tasks(db: Arc<PgPool>, task_ids: Vec<i32>) -> Result<u64, SvixError> {
        if task_ids.len() <= 0 {
            return Ok(0);
        }

        let tids: Vec<String> = task_ids.iter().map(|x| x.to_string()).collect();
        let tids: String = tids.join(",");
        let query_raw = format!("DELETE FROM Tasks WHERE id in ({})", &tids);

        let delete_query = sqlx::query(&query_raw);

        match db.execute(delete_query).await {
            Ok(res) => Ok(res.rows_affected()),
            Err(err) => Err(SvixError::Unknown { message: err.to_string() })
        }
    }
}