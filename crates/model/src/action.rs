use crate::common::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Action {
  Welcome,
  SetBio { text: String },
  AcceptCandidate { id: UserId },
  RejectCandidate { id: UserId },
}
