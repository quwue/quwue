use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::{env, fs, path::Path};

#[tokio::main]
async fn main() {
  #![allow(clippy::semicolon_if_nothing_returned)]

  let db_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("db.sqlite");

  fs::remove_file(&db_path).ok();

  let db_url = db_url::db_url(&db_path).unwrap();

  Sqlite::create_database(&db_url).await.unwrap();

  let pool = SqlitePool::connect(&db_url).await.unwrap();

  sqlx::migrate!("./migrations").run(&pool).await.unwrap();

  println!("cargo:rustc-env=DATABASE_URL={}", db_url);

  println!("cargo:rerun-if-changed=migrations");
}
