use crate::common::*;

pub(crate) trait Value: Sized {
  type Err;
  type Storage;

  fn store(self) -> Self::Storage;

  fn load(storage: Self::Storage) -> Result<Self, Self::Err>;
}

impl Value for u64 {
  type Err = Infallible;
  type Storage = i64;

  fn store(self) -> Self::Storage {
    i64::from_le_bytes(self.to_le_bytes())
  }

  fn load(storage: Self::Storage) -> Result<Self, Self::Err> {
    Ok(Self::from_le_bytes(storage.to_le_bytes()))
  }
}

impl Value for Prompt {
  type Err = Error;
  type Storage = (i64, Option<i64>);

  fn store(self) -> Self::Storage {
    match self {
      Self::Bio => (0, None),
      Self::Candidate { id } => (1, Some(id.store())),
      Self::Match { id } => (2, Some(id.store())),
      Self::ProfileImage => (3, None),
      Self::Quiescent => (4, None),
      Self::Welcome => (5, None),
    }
  }

  fn load(storage: Self::Storage) -> Result<Self, Self::Err> {
    Ok(match storage {
      (0, None) => Self::Bio,
      (1, Some(id)) => Self::Candidate {
        id: UserId::load(id).unwrap_infallible(),
      },
      (2, Some(id)) => Self::Match {
        id: UserId::load(id).unwrap_infallible(),
      },
      (3, None) => Self::ProfileImage,
      (4, None) => Self::Quiescent,
      (5, None) => Self::Welcome,
      _ => todo!(),
    })
  }
}

impl Value for UserId {
  type Err = Infallible;
  type Storage = i64;

  fn store(self) -> Self::Storage {
    self.0.store()
  }

  fn load(storage: Self::Storage) -> Result<Self, Self::Err> {
    Ok(UserId(u64::load(storage).unwrap_infallible()))
  }
}

impl Value for MessageId {
  type Err = Infallible;
  type Storage = i64;

  fn store(self) -> Self::Storage {
    self.0.store()
  }

  fn load(storage: Self::Storage) -> Result<Self, Self::Err> {
    Ok(MessageId(u64::load(storage).unwrap_infallible()))
  }
}
