use crate::common::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Action {
  Welcome,
  SetBio { text: String },
  SetProfileImage { url: Url },
  AcceptCandidate { id: UserId },
  RejectCandidate { id: UserId },
}
