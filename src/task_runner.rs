use std::{time::Duration};

use actix_web::{rt};
use chrono::Utc;
use sqlx::{PgPool};

use crate::{db::DbOps};

pub struct TaskRunner {
    pub run_count: u128,
    pub db: PgPool,
    pub interval: u64
}

impl TaskRunner {
    pub fn new(db: PgPool) -> Self {
        Self { run_count: 0, db: db, interval: 5 }
    }

    pub async fn start(&mut self) {
        loop {
            let cur_time_utc = Utc::now();
            let max_time_checked = cur_time_utc + chrono::Duration::days(5);
            println!("\t[{}] Checking tasks before {}", self.run_count, &max_time_checked);
            
            // Get all tasks within a certain time (currently 5 days for testing but move to 5 seconds)
            let Ok(tasks) = DbOps::get_all_tasks_before(self.db.clone().into(), max_time_checked).await else {
                rt::time::sleep(Duration::from_secs(4)).await;
                continue;
            };

            let mut tasks_to_delete: Vec<i32> = Vec::new();
            // Execute the tasks
            for task in tasks {
                println!("\t\tTask: {:0>5} {}", task.id, Into::<String>::into(task.task_type));
                task.execute().await;
                tasks_to_delete.push(task.id);
            }

            // Delete executed tasks (In the future, have checks if they need to be rescheduled)
            match DbOps::delete_tasks(self.db.clone().into(), tasks_to_delete).await {
                Ok(_rows_affected) => {},
                Err(err) => println!("DELETE TASKS ERRPR: {}", Into::<String>::into(err))
            }

            // Sleep for sometime less than 5 seconds (so we don't miss tasks that are in the grey area between the two executions)
            rt::time::sleep(Duration::from_secs(4)).await;
            self.run_count += 1;
        }
    }
}