#![cfg_attr(test, allow(clippy::panic))]

#[cfg(test)]
mod expect_var;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod test_bot;
#[cfg(test)]
mod test_dispatcher;
#[cfg(test)]
mod test_event;
#[cfg(test)]
mod test_id;
#[cfg(test)]
mod test_message;
#[cfg(test)]
mod test_run_id;
#[cfg(test)]
mod test_user;

mod async_static;
mod bot;
mod common;
mod error;
mod logging;
mod rate_limit;
mod runtime;
mod test_name;
mod test_user_id;

fn main() {
  use crate::common::*;
  if let Err(error) = Bot::main() {
    use ansi_term::{Color, Style};
    let red = Style::new().fg(Color::Red).bold();
    let bold = Style::new().bold();
    eprintln!("{}: {}", red.paint("error"), bold.paint(error.to_string()));
    process::exit(1);
  }
}
