use std::fmt;
use sqlx::{Executor, MySqlPool};


#[derive(Debug, PartialEq, Eq)]
#[derive(sqlx::FromRow)]
pub struct Job {
    id: String,
    created_at: i64,
    pub name: std::string::String,
}

#[derive(Debug)]
pub enum JobStatus {
  Pending,
  Processing,
  Done,
  // Errored
}

impl fmt::Display for JobStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{:?}", self)
  }
}

pub async fn get_next_job(pool: &MySqlPool) -> Result<Job, sqlx::Error> {

  println!("Querying jobs...");
  let job = sqlx::query_as!(
    Job,
    r#"SELECT id, created_at, name as "name!" from job
          where status = 'Pending'
          order by created_at asc
          LIMIT 1"#)
    .fetch_one(pool)
    .await?;

  println!("Found job of type {}, (id={})", job.name, job.id);

  return Ok(job);
}

pub async fn update_job_status(pool: &MySqlPool, job: &Job, status: JobStatus) {
  let result = pool.execute(sqlx::query(
    r"UPDATE job
          SET status = ?
          WHERE id = ?"
        )
        .bind(status.to_string())
        .bind(&job.id)
        ).await; 

  match result {
    Ok(_) => println!("Successfully updated job status on record with id: {}", job.id),
    Err(e) => println!("Failed to update job status on record with id: {}, error: {}", job.id, e)
  }
}
