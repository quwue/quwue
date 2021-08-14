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
mod test_name;
#[cfg(test)]
mod test_user;

mod arguments;
mod async_static;
mod bot;
mod common;
mod error;
mod logging;
mod rate_limit;
mod response_future_ext;
mod runtime;
mod test_id;
mod test_message;
mod test_run_id;
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
