use cradle::prelude::*;
use std::{path::Path, process::Command};

fn main() {
  if !run_output!(%"id --user quwue") {
    run!(%"useradd --system quwue");
  }

  let StdoutTrimmed(root_exists) = run_output!(
    "psql",
    "postgres",
    "-tAc",
    "SELECT 1 FROM pg_roles WHERE rolname='root'"
  );

  if root_exists != "1" {
    run!(%"sudo -u postgres createuser root");
  }

  run!(%"sudo -u postgres psql postgres -c", "ALTER user root createdb");

  let StdoutTrimmed(quwue_exists) = run_output!(
    "psql",
    "postgres",
    "-tAc",
    "SELECT 1 FROM pg_roles WHERE rolname='quwue'"
  );

  if quwue_exists != "1" {
    run!(%"sudo -u postgres createuser quwue");
  }

  run!(%"sudo -u postgres psql postgres -c", "ALTER user quwue createdb");

  run!(
    %"cargo install --root /usr/local --path tmp/quwue --force"
  );

  run!(
    %"cp tmp/quwue/quwue.service /etc/systemd/system/quwue.service"
  );

  run!(
    %"chmod 664 /etc/systemd/system/quwue.service"
  );

  if !Path::new("/etc/systemd/system/quwue.service.d/override.conf").is_file() {
    let status = Command::new("systemctl")
      .args(&["edit", "quwue"])
      .status()
      .unwrap();

    if !status.success() {
      panic!("Command `systemctl edit quwue` failed: {}", status);
    }
  }

  run!(
    %"systemctl daemon-reload"
  );

  run!(
    %"systemctl restart quwue"
  );
}
