use {cradle::prelude::*, std::process::Command, structopt::StructOpt};

#[derive(StructOpt)]
struct Arguments {
  #[structopt(long)]
  host: String,
}

impl Arguments {
  fn run(&self) {
    self.ssh("apt-get update");

    self.ssh("apt-get install --yes build-essential postgresql");

    eprintln!(
      "Set IPv4 and IPv6 local connections to trust in /etc/postgresql/13/main/pg_hba.conf"
    );
    eprintln!();
    eprintln!("# IPv4 local connections:");
    eprintln!("host    all             all             127.0.0.1/32            trust");
    eprintln!("# IPv6 local connections:");
    eprintln!("host    all             all             ::1/128                 trust");

    Command::new("bash").args(&["-c", "read"]).output().unwrap();

    self.ssh_check(
      "[[ -f rustup.sh ]]",
      "curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh",
    );

    self.ssh_check("[[ -f .cargo/bin/cargo ]]", "sh rustup.sh -y");

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
    run!("ssh", format!("root@{}", self.host), Split(command));
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
