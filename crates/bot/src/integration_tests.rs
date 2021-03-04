use crate::common::*;

fn test<F: Future>(f: F) -> F::Output {
  static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| runtime::init().expect("Failed to initialize test runtime."));

  RUNTIME.block_on(f)
}

#[instrument]
#[test]
#[ignore]
fn welcome_initial_response() {
  test(async {
    let mut user = test_user!().await;

    user.send_message("hi").await;
    user.expect_prompt(Prompt::Welcome).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn welcome_bad_message() {
  test(async {
    let mut user = test_user!().await;

    user.send_message("hi").await;
    user.expect_prompt(Prompt::Welcome).await;
    user.send_message("foo").await;
    user.expect_prompt(Prompt::Welcome).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn welcome_confirm_message() {
  test(async {
    let mut user = test_user!().await;

    user.send_message("hi").await;
    user.expect_prompt(Prompt::Welcome).await;
    user.send_message("ok").await;
    user.expect_prompt(Prompt::Bio).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn welcome_confirm_react() {
  test(async {
    let mut user = test_user!().await;

    user.send_message("hi").await;
    let id = user.expect_prompt(Prompt::Welcome).await;
    user.send_reaction(id, Emoji::ThumbsUp).await;
    user.expect_prompt(Prompt::Bio).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn set_bio() {
  test(async {
    let mut user = test_user!().await;

    user.send_message("hi").await;
    let id = user.expect_prompt(Prompt::Welcome).await;
    user.send_reaction(id, Emoji::ThumbsUp).await;
    user.expect_prompt(Prompt::Bio).await;
    user.send_message("my bio!").await;
    user.expect_prompt(Prompt::Quiescent).await;
  })
}
