use reqwest::{self, StatusCode};
use sqlx::{MySqlPool};

use crate::job::{Job, JobStatus, update_job_status};

pub async fn process_job(pool: &MySqlPool, job: &Job) -> u64 {
  let api_link = "https://api.darkowl.dev/job_status";
  match make_request(api_link) {
      Err(e) => unexpected_error_handler(e, pool, job).await,
      Ok(status)  => {
        match status {
          StatusCode::OK => success_handler(pool, job).await,
          _ => error_handler(pool, job).await

        }
      }
  }
}
// Response is not a json object conforming to the Simple struct
fn make_request(link: &str) -> Result<StatusCode, reqwest::Error> {
  let status = reqwest::get(link)?.status();
  println!("Status code was {}", status);
  Ok(status)
}

async fn error_handler(pool: &MySqlPool, job: &Job) -> u64 {
  println!("The API was not available");
  update_job_status(pool, job, JobStatus::Pending).await;
  return 1;
}

async fn unexpected_error_handler(e: reqwest::Error, pool: &MySqlPool, job: &Job) -> u64 {
  println!("Unexpected error: {}", e);
  update_job_status(pool, job, JobStatus::Pending).await;
  return 1;
}

async fn success_handler(pool: &MySqlPool, job: &Job) -> u64 {
  update_job_status(pool, job, JobStatus::Done).await;
  return 0;
}

pub async fn begin_processing(pool: &MySqlPool, job: Job) -> Result<Job, sqlx::Error> {
  
  update_job_status(pool, &job, JobStatus::Processing).await;

  process_job(pool, &job).await;

  println!("Finished {}", job.name);

  return Ok(job);
}