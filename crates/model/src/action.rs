use crate::common::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Action {
  AcceptCandidate { id: UserId },
  DismissMatch { id: UserId },
  RejectCandidate { id: UserId },
  SetBio { text: String },
  Welcome
}
