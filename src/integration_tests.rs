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
fn complete_profile() {
  test(async {
    let mut bot = test_bot!().await;
    let mut user = bot.new_user().await;

    user.setup().await;
    user.expect_prompt(Prompt::Quiescent).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn multi_user_message() {
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
fn multi_user_react() {
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

#[instrument]
#[test]
#[ignore]
fn multi_user_candidate() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;
    b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn multi_user_candidate_accept() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;
    let mut c = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;
    let id = b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    b.send_reaction(id, Emoji::ThumbsDown).await;

    c.setup().await;
    let id = c.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    c.send_reaction(id, Emoji::ThumbsUp).await;

    let id = c.expect_prompt(Prompt::Candidate { id: b.id() }).await;
    c.send_reaction(id, Emoji::ThumbsUp).await;

    c.expect_prompt(Prompt::Quiescent).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn candidate__and_match_interrupts() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;

    let id = b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    b.send_reaction(id, Emoji::ThumbsUp).await;
    b.expect_prompt(Prompt::Quiescent).await;

    let id = a.expect_prompt(Prompt::Candidate { id: b.id() }).await;
    a.send_reaction(id, Emoji::ThumbsUp).await;
    a.expect_prompt(Prompt::Match { id: b.id() }).await;

    b.expect_prompt(Prompt::Match { id: a.id() }).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn match_prompt() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;

    let id = b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    b.send_reaction(id, Emoji::ThumbsUp).await;
    b.expect_prompt(Prompt::Quiescent).await;

    let id = a.expect_prompt(Prompt::Candidate { id: b.id() }).await;
    a.send_reaction(id, Emoji::ThumbsUp).await;

    let prompt = Prompt::Match { id: b.id() };
    assert!(bot
      .db()
      .prompt_text_outside_update_transaction(prompt)
      .await
      .contains("b's bio!"));
    a.expect_prompt(prompt).await;

    let prompt = Prompt::Match { id: a.id() };
    assert!(bot
      .db()
      .prompt_text_outside_update_transaction(prompt)
      .await
      .contains("a's bio!"));
    b.expect_prompt(prompt).await;

    a.send_message("foo").await;
    a.expect_prompt(Prompt::Match { id: b.id() }).await;

    a.send_message("ok").await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.send_message("foo").await;
    let b_prompt_id = b.expect_prompt(Prompt::Match { id: a.id() }).await;

    b.send_reaction(b_prompt_id, Emoji::ThumbsUp).await;
    b.expect_prompt(Prompt::Quiescent).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn accept_candidate_with_message() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;

    b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    b.send_message("yes").await;
    b.expect_prompt(Prompt::Quiescent).await;

    a.expect_prompt(Prompt::Candidate { id: b.id() }).await;
    a.send_message("y").await;

    let prompt = Prompt::Match { id: b.id() };
    assert!(bot
      .db()
      .prompt_text_outside_update_transaction(prompt)
      .await
      .contains("b's bio!"));
    a.expect_prompt(prompt).await;

    let prompt = Prompt::Match { id: a.id() };
    assert!(bot
      .db()
      .prompt_text_outside_update_transaction(prompt)
      .await
      .contains("a's bio!"));
    b.expect_prompt(prompt).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn reject_candidate_with_no() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;

    b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    b.send_message("no").await;
    b.expect_prompt(Prompt::Quiescent).await;

    a.expect_nothing().await;
  })
}

#[instrument]
#[test]
#[ignore]
fn reject_candidate_with_n() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;

    b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    b.send_message("n").await;
    b.expect_prompt(Prompt::Quiescent).await;

    a.expect_nothing().await;
  })
}

#[instrument]
#[test]
#[ignore]
fn candidate_hidden_after_rejection() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;

    b.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    b.send_message("yes").await;
    b.expect_prompt(Prompt::Quiescent).await;

    a.expect_prompt(Prompt::Candidate { id: b.id() }).await;
    a.send_message("no").await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.expect_nothing().await;
  })
}

#[instrument]
#[test]
#[ignore]
fn dont_show_users_as_candidates_when_they_have_pending_candidate_prompts() {
  test(async {
    let mut bot = test_bot!().await;
    let mut a = bot.new_user().await;
    let mut b = bot.new_user().await;
    let mut c = bot.new_user().await;

    a.setup().await;
    a.expect_prompt(Prompt::Quiescent).await;

    b.setup().await;
    b.expect_prompt(Prompt::Candidate { id: a.id() }).await;

    c.setup().await;
    c.expect_prompt(Prompt::Candidate { id: a.id() }).await;
    c.send_message("yes").await;
    c.expect_prompt(Prompt::Quiescent).await;

    a.expect_prompt(Prompt::Candidate { id: c.id() }).await;
    a.send_message("yes").await;
    a.expect_prompt(Prompt::Match { id: c.id() }).await;

    c.expect_prompt(Prompt::Match { id: a.id() }).await;
  })
}
