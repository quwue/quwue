use cradle::prelude::*;
use std::{path::Path, process::Command};

fn main() {
  if !run_output!(%"id --user quwue") {
    run!(%"useradd --system quwue");
  }

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
