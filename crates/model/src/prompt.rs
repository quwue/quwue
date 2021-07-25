use crate::common::*;

use Emoji::*;
use Prompt::*;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Prompt {
  Bio,
  Candidate { id: UserId },
  Match { id: UserId },
  ProfileImage,
  Quiescent,
  Welcome,
}

impl Prompt {
  pub fn reactions(self) -> Vec<Emoji> {
    match self {
      Welcome => vec![ThumbsUp],
      Candidate { .. } => vec![ThumbsUp, ThumbsDown],
      Bio | ProfileImage | Quiescent | Match { .. } => vec![],
    }
  }

  pub fn quiescent(self) -> bool {
    self == Quiescent
  }
}
