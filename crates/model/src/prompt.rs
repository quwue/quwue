use crate::common::*;

use Emoji::*;
use Prompt::*;

/// NB. Prompt priority is determined by enum variant order.
#[derive(Debug, EnumDiscriminants, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[strum_discriminants(
  repr(u64),
  derive(TryFromPrimitive, PartialOrd, Ord, EnumIter),
  name(PromptDiscriminant)
)]
pub enum Prompt {
  Welcome,
  Bio,
  Quiescent,
  Candidate { id: UserId },
  Match { id: UserId },
}

impl Prompt {
  pub fn reactions(self) -> &'static [Emoji] {
    match self {
      Candidate { .. } => &[ThumbsUp, ThumbsDown],
      Match { .. } | Welcome => &[ThumbsUp],
      Bio | Quiescent => &[],
    }
  }

  pub fn quiescent(self) -> bool {
    self == Quiescent
  }

  pub fn discriminant(self) -> PromptDiscriminant {
    self.into()
  }

  pub fn cannot_interrupt(self, discriminant: PromptDiscriminant) -> bool {
    self.discriminant() <= discriminant
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use strum::IntoEnumIterator;

  #[test]
  fn priority() {
    use PromptDiscriminant::*;

    let mut discriminants = PromptDiscriminant::iter().collect::<Vec<PromptDiscriminant>>();
    discriminants.sort();

    assert_eq!(discriminants, vec![
      Welcome, Bio, Quiescent, Candidate, Match
    ]);
  }
}
