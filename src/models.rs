use std::time::Duration;

use actix_web::rt;
use chrono::{DateTime, TimeZone, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::common::SvixError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskType {
    Foo,
    Bar,
    Baz,
}

impl Into<String> for TaskType {
    fn into(self) -> String {
        match self {
            TaskType::Foo => String::from("Foo"),
            TaskType::Bar => String::from("Bar"),
            TaskType::Baz => String::from("Baz"),
        }
    }
}

impl TryFrom<String> for TaskType {
    type Error = SvixError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Foo" => Ok(TaskType::Foo),
            "Bar" => Ok(TaskType::Bar),
            "Baz" => Ok(TaskType::Baz),
            _ => Err(SvixError::new(&format!(
                "Cannot convert '{}' to TaskType",
                value.as_str()
            ))),
        }
    }
}

#[derive(FromRow, Debug, Clone, Copy, Serialize)]
pub struct Task {
    pub id: i32,
    pub task_type: TaskType,
    pub scheduled_for: DateTime<Utc>,
    pub repeat: bool,
    pub last_run: Option<DateTime<Utc>>,
}

impl Task {
    // TODO: Shouldn't be here
    pub fn get_date(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> Result<DateTime<Utc>, String> {
        match Utc.with_ymd_and_hms(year, month, day, hour, min, sec) {
            chrono::LocalResult::None => Err(String::from("No conversion exists.")),
            chrono::LocalResult::Single(val) => Ok(val),
            chrono::LocalResult::Ambiguous(_, _) => Err(String::from("Ambigious conversion")),
        }
    }

    pub async fn execute(&self) {
        match self.task_type {
            TaskType::Foo => {
                rt::time::sleep(Duration::from_secs(3)).await;
                println!("Foo {}", self.id);
            }
            TaskType::Bar => {
                // TODO: should be moved to task runner?
                let client = awc::Client::default();
                let request = client.get("https://www.whattimeisitrightnow.com/").insert_header(("User-Agent", "Mozilla/5.0 (compatible; MSIE 9.0; Windows NT 6.1; WOW64; Trident/5.0)"));
                let result = request.send().await;
                match result {
                    Ok(resp) => println!("Bar {}", resp.status().to_string()),
                    Err(err) => println!("BAR ERROR: {}", err.to_string()),
                }
            }
            TaskType::Baz => {
                // TODO: should be saved in task runner
                let mut rng = rand::thread_rng();
                let rand_num = rng.gen_range(0..=343);
                println!("Baz {}", rand_num)
            }
        };
    }
}

impl TryFrom<PgRow> for Task {
    type Error = SvixError;

    fn try_from(val: PgRow) -> Result<Self, Self::Error> {
        let Ok(id) = val.try_get::<i32, _>("id") else { return Err(SvixError::new("Could not parse 'id' field of Task")); };
        let Ok(task_type) = val.try_get::<String, _>("task_type") else { return Err(SvixError::new("Could not parse 'task_type' field of Task")); };
        let Ok(scheduled_for) = val.try_get::<DateTime<Utc>, _>("scheduled_for") else { return Err(SvixError::new("Could not parse 'scheduled_for' field of Task")); };
        let Ok(repeat) = val.try_get::<bool, _>("repeat") else { return Err(SvixError::new("Could not parse 'repeat' field of Task")); };
        let Ok(last_run) = val.try_get::<Option<DateTime<Utc>>, _>("last_run") else { return Err(SvixError::new("Could not parse 'last_run' field of Task")); };

        let Ok(task_type) = TaskType::try_from(task_type) else { return Err(SvixError::new("Unable to convert to TaskType")); };

        Ok(Task {
            id: id,
            task_type: task_type,
            scheduled_for: scheduled_for,
            repeat: repeat,
            last_run: last_run,
        })
    }
}
