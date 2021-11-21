use {
  cradle::prelude::*,
  std::{path::Path, process::Command},
};

fn main() {
  if !run_output!(LogCommand, %"id --user quwue") {
    run!(LogCommand, %"useradd --system quwue");
  }

  let (StdoutTrimmed(stdout), Status(status)) = run_output!(
    LogCommand,
    "psql",
    "postgres",
    "-tAc",
    "SELECT 1 FROM pg_roles WHERE rolname='root'"
  );

  if !status.success() || stdout != "1" {
    run!(LogCommand, %"sudo -Hiu postgres createuser root");
  }

  run!(LogCommand, %"sudo -Hiu postgres psql postgres -c", "ALTER user root createdb");

  let (StdoutTrimmed(stdout), Status(status)) = run_output!(
    LogCommand,
    "psql",
    "postgres",
    "-tAc",
    "SELECT 1 FROM pg_roles WHERE rolname='quwue'"
  );

  if !status.success() || stdout != "1" {
    run!(LogCommand, %"sudo -Hiu postgres createuser quwue");
  }

  run!(LogCommand, %"sudo -Hiu postgres psql postgres -c", "ALTER user quwue createdb");

  run!(
    LogCommand, %"cargo install --root /usr/local --path tmp/quwue --force"
  );

  run!(
    LogCommand, %"cp tmp/quwue/quwue.service /etc/systemd/system/quwue.service"
  );

  run!(
    LogCommand, %"chmod 664 /etc/systemd/system/quwue.service"
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
    LogCommand, %"systemctl daemon-reload"
  );

  run!(
    LogCommand, %"systemctl restart quwue"
  );
}
