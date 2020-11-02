#![allow(clippy::panic, clippy::unwrap_used)]

use std::{env, fs, path::Path, process::Command};

fn sqlx(args: &[&str], db_url: &str) {
  let output = Command::new("sqlx")
    .args(args)
    .env("DATABASE_URL", db_url)
    .output()
    .unwrap();

  if !output.status.success() {
    panic!("Command `{}` failed: {:?}", args.join(" "), output);
  }
}

fn main() {
  let db_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("db.sqlite");
  let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

  fs::remove_file(&db_path).ok();

  sqlx(&["database", "create"], &db_url);
  sqlx(&["migrate", "run"], &db_url);

  println!("cargo:rustc-env=DATABASE_URL={}", db_url);

  for result in fs::read_dir("migrations").unwrap() {
    let entry = result.unwrap();
    println!("cargo:rerun-if-changed={}", entry.path().display());
  }
}
