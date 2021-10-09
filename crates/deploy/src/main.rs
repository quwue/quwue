use cradle::prelude::*;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Arguments {
  #[structopt(long)]
  host: String,
}

impl Arguments {
  fn run(&self) {
    self.ssh("apt-get update");

    self.ssh("apt-get install --yes build-essential");

    self.ssh_check(
      "[[ -f rustup.sh ]]",
      "curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh",
    );

    self.ssh_check("[[ -d .cargo ]]", "sh rustup.sh -y");

    self.ssh_check("[[ -d tmp ]]", "mkdir tmp");

    run!(
      %"rsync --progress --verbose --compress --recursive --links --perms --times",
      %"--exclude .git --exclude target --exclude .vagrant --delete .",
      format!("root@{}:tmp/quwue/", self.host)
    );

    let status = Command::new("ssh")
      .args(&[
        "-t",
        &format!("root@{}", self.host),
        "~/.cargo/bin/cargo",
        "run",
        "--manifest-path",
        "tmp/quwue/Cargo.toml",
        "--package",
        "setup",
      ])
      .status()
      .unwrap();

    if !status.success() {
      panic!("setup failed: {}", status);
    }
  }

  fn ssh(&self, command: &'static str) {
    run!("ssh", format!("root@{}", self.host), Split(command))
  }

  fn ssh_check(&self, check: &'static str, command: &'static str) {
    let Status(status) = run_output!("ssh", format!("root@{}", self.host), Split(check));

    if !status.success() {
      run!("ssh", format!("root@{}", self.host), Split(command))
    }

    let Status(status) = run_output!("ssh", format!("root@{}", self.host), Split(check));

    if !status.success() {
      panic!(
        "Check `{}` failed after running command `{}",
        check, command
      );
    }
  }
}

fn main() {
  Arguments::from_args().run();
}
