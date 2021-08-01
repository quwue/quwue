use crate::common::*;

use Emoji::*;
use Prompt::*;

#[derive(Debug, EnumDiscriminants, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[strum_discriminants(repr(u64), derive(TryFromPrimitive), name(PromptDiscriminant))]
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

  pub fn discriminant(self) -> PromptDiscriminant {
    self.into()
  }
}
