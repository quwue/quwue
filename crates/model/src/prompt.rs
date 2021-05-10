use crate::common::*;

use Emoji::*;
use Prompt::*;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Prompt {
  Welcome,
  Quiescent,
  Candidate { id: UserId },
  Match { id: UserId },
  Bio,
  ProfileImage,
}

impl Prompt {
  pub fn reactions(self) -> Vec<Emoji> {
    match self {
      Welcome => vec![ThumbsUp],
      Quiescent => vec![],
      Candidate { .. } => vec![ThumbsDown, ThumbsUp],
      Bio => vec![],
      ProfileImage => vec![],
      Match { .. } => vec![],
    }
  }

  pub fn quiescent(self) -> bool {
    self == Quiescent
  }
}
