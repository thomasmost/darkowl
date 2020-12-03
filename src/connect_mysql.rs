use sqlx::MySqlPool;

pub async fn get_pool(db_url: &String) -> Result<MySqlPool, sqlx::Error>  {
  MySqlPool::connect(db_url).await
}
