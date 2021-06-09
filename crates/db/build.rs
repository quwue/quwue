use camino::Utf8PathBuf;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::{convert::TryFrom, env, fs, path::Path};

#[tokio::main]
async fn main() {
  let db_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("db.sqlite");
  let db_path = Utf8PathBuf::try_from(db_path).unwrap();

  let db_url = if cfg!(windows) {
    format!("sqlite:///{}", db_path.as_str().replace('\\', "\\\\"))
  } else {
    format!("sqlite:{}", db_path)
  };

  Sqlite::create_database(&db_url).await.unwrap();

  let pool = SqlitePool::connect(&db_url).await.unwrap();

  sqlx::migrate!("./migrations").run(&pool).await.unwrap();

  println!("cargo:rustc-env=DATABASE_URL={}", db_url);

  for result in fs::read_dir("migrations").unwrap() {
    let entry = result.unwrap();
    println!("cargo:rerun-if-changed={}", entry.path().display());
  }
}
