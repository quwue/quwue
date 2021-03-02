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
    let mut expect = expect!().await;

    expect.send("hi").await;
    expect.prompt(Prompt::Welcome).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn welcome_bad_message() {
  test(async {
    let mut expect = expect!().await;

    expect.send("hi").await;
    expect.prompt(Prompt::Welcome).await;
    expect.send("foo").await;
    expect.prompt(Prompt::Welcome).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn welcome_confirm_message() {
  test(async {
    let mut expect = expect!().await;

    expect.send("hi").await;
    expect.prompt(Prompt::Welcome).await;
    expect.send("ok").await;
    expect.prompt(Prompt::Bio).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn welcome_confirm_react() {
  test(async {
    let mut expect = expect!().await;

    expect.send("hi").await;
    let id = expect.prompt(Prompt::Welcome).await;
    expect.react(id, Emoji::ThumbsUp).await;
    expect.prompt(Prompt::Bio).await;
  })
}

#[instrument]
#[test]
#[ignore]
fn set_bio() {
  test(async {
    let mut expect = expect!().await;

    expect.send("hi").await;
    let id = expect.prompt(Prompt::Welcome).await;
    expect.react(id, Emoji::ThumbsUp).await;
    expect.prompt(Prompt::Bio).await;
    expect.send("my bio!").await;
    expect.prompt(Prompt::Quiescent).await;
  })
}

#[cfg(feature = "foo")]
#[instrument]
#[test]
#[ignore]
fn candidate() {
  test(async {
    let mut a = expect!().await;
    // a.send("hi").await;
    // let id = a.prompt(Prompt::Welcome).await;
    // a.react(id, Emoji::ThumbsUp).await;
    // a.prompt(Prompt::Quiescent).await;

    // let mut b = expect!().await;
    // b.send("hi").await;
    // let id = b.prompt(Prompt::Welcome).await;
    // b.react(id, Emoji::ThumbsUp).await;
    // b.prompt(Prompt::Candidate).await;

    // a.prompt(Prompt::Candidate).await;
  })
}
