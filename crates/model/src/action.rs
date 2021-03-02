#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Action {
  Welcome,
  SetBio { text: String },
}
