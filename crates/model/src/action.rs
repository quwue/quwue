use crate::common::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Action {
  AcceptCandidate { id: UserId },
  DeclineCandidate { id: UserId },
  DismissMatch { id: UserId },
  SetBio { text: String },
  Welcome,
}
