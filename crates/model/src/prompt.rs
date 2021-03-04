use crate::common::*;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use Emoji::*;
use Prompt::*;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, IntoPrimitive, TryFromPrimitive, Copy, Clone)]
#[repr(u64)]
pub enum Prompt {
  Welcome,
  Quiescent,
  Candidate,
  Bio,
  ProfileImage,
}

impl Prompt {
  pub fn text(self) -> String {
    match self {
      Welcome => format!(
        concat!(
          "Hi!\n",
          "Quwue is a bot that matches you with other Discord users.\n",
          "Your Discord tag will only be revealed to matches.\n",
          "To start, you'll need to set up your profile.\n",
          "Hit the {} emoji or type `ok` to continue.",
        ),
        Emoji::ThumbsUp.markup()
      ),
      Quiescent => "You've seen all available matches. We'll message you when we have new matches \
                    to show you!"
        .into(),

      Candidate => "TODO".into(),
      Bio => "Please enter a bio to show to other users.".into(),
      ProfileImage => "Please enter your profile photo.".into(),
    }
  }

  pub fn reactions(self) -> Vec<Emoji> {
    match self {
      Welcome => vec![ThumbsUp],
      Quiescent => vec![],
      Candidate => vec![ThumbsDown, ThumbsUp],
      Bio => vec![],
      ProfileImage => vec![],
    }
  }

  pub fn quiescent(self) -> bool {
    self == Quiescent
  }
}
