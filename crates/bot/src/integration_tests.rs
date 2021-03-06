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
    let mut bot = test_bot!().await;
    let mut user = bot.new_user().await;

    user.send_message("hi").await;
    user.expect_prompt(Prompt::Welcome).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn welcome_bad_message() {
  test(async {
    let mut bot = test_bot!().await;
    let mut user = bot.new_user().await;

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
    let mut bot = test_bot!().await;
    let mut user = bot.new_user().await;

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
    let mut bot = test_bot!().await;
    let mut user = bot.new_user().await;

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
    let mut bot = test_bot!().await;
    let mut user = bot.new_user().await;

    user.send_message("hi").await;
    let id = user.expect_prompt(Prompt::Welcome).await;
    user.send_reaction(id, Emoji::ThumbsUp).await;
    user.expect_prompt(Prompt::Bio).await;
    user.send_message("my bio!").await;
    user.expect_prompt(Prompt::Quiescent).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn multi_user_message_test() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.send_message("hi").await;
    a.expect_prompt(Prompt::Welcome).await;
    a.send_message("ok").await;
    a.expect_prompt(Prompt::Bio).await;
    b.send_message("hi").await;
    b.expect_prompt(Prompt::Welcome).await;
    b.send_message("ok").await;
    b.expect_prompt(Prompt::Bio).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn multi_user_react_test() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.send_message("hi").await;
    let id = a.expect_prompt(Prompt::Welcome).await;
    a.send_reaction(id, Emoji::ThumbsUp).await;
    a.expect_prompt(Prompt::Bio).await;

    b.send_message("hi").await;
    let id = b.expect_prompt(Prompt::Welcome).await;
    b.send_reaction(id, Emoji::ThumbsUp).await;
    b.expect_prompt(Prompt::Bio).await;
  })
}
