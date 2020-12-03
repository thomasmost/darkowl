extern crate dotenv;
extern crate sqlx;
#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;

mod connect_mysql;
mod job;
mod processor;
use dotenv::dotenv;
use lambda::error::HandlerError;
use job::{Job, get_next_job};
use std::error::Error;
use std::env;
use futures::executor::block_on;


async fn handle_connection_error (e: sqlx::Error) {
    println!("Connection failed: {:?}", e);
}

fn handle_query_error (e: sqlx::Error) -> std::result::Result<Job, sqlx::Error> {
    println!("Query failed: {:?}", e);
    return Err(e);
}

async fn run_with_pool(pool: &sqlx::mysql::MySqlPool) {
    let job_result = get_next_job(&pool).await;

    let processing_result = match job_result {
        Ok(job) => processor::begin_processing(&pool, job).await,
        Err(e) => handle_query_error(e),
    };
    match processing_result {
        Ok(_) => println!("Completed operations successfully!"),
        Err(e) => println!("Sorry, processing exited unexpectedly due to the following error: {}", e),
    };
}

#[derive(Deserialize, Clone)]
struct CloudwatchLambdaEvent {
    id: String,
    source: String,
}

// #[derive(Deserialize, Clone)]
// struct CustomLambdaEvent {
//     #[serde(rename = "externalRequestId")]
//     external_request_id: String,
//     message: String,
// }

#[derive(Serialize, Clone)]
struct LambdaOutput {
    message: String,
}

fn my_handler(e: CloudwatchLambdaEvent, c: lambda::Context) -> Result<LambdaOutput, HandlerError> {

    println!("~h00t~ Source={}, Id={}", e.source, e.id);
    let url_result = &env::var("DATABASE_URL");
    
    match url_result {
        Ok(url) => {
            block_on(run(url));
    
            let message: String = "Success".to_string();

            Ok(LambdaOutput { message })
        },
        Err(_e) => Err(c.new_error("Missing DATABASE_URL"))
    }

}

async fn run(url: &String) {
    let connection_result = connect_mysql::get_pool(url).await;

    match connection_result {
        Ok(pool) => run_with_pool(&pool).await,
        Err(e) => handle_connection_error(e).await,
    };
}

// fn main() {
//     dotenv().ok();
//     println!("Let's go!");
//     let url_result = &env::var("DATABASE_URL");
    
//     match url_result {
//             Ok(url) => block_on(run(url)),
//             Err(_) => println!("Missing DATABASE_URL"),
//         };

// }


fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    lambda!(my_handler);

    Ok(())
}

