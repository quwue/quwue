use {
  sqlx::{migrate::MigrateDatabase, PgPool, Postgres},
  std::time::SystemTime,
};

#[tokio::main]
async fn main() {
  let db_url = db_url::db_url(&format!(
    "quwue-build-{}",
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  ));

  Postgres::create_database(&db_url).await.unwrap();

  let pool = PgPool::connect(&db_url).await.unwrap();

  sqlx::migrate!("./migrations").run(&pool).await.unwrap();

  println!("cargo:rustc-env=DATABASE_URL={}", db_url);

  println!("cargo:rerun-if-changed=migrations");
}
