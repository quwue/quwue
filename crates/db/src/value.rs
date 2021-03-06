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
    let payload = match self {
      Self::Bio | Self::Quiescent | Self::Welcome => None,
      Self::Candidate { id } | Self::Match { id } => Some(id.store()),
    };

    ((self.discriminant() as u64).store(), payload)
  }

  fn load((discriminant, payload): Self::Storage) -> Result<Self, Self::Err> {
    use PromptDiscriminant::*;

    let discriminant = PromptDiscriminant::load(discriminant)?;

    match (discriminant, payload) {
      (Bio, None) => Ok(Self::Bio),
      (Candidate, Some(id)) => Ok(Self::Candidate {
        id: UserId::load(id).unwrap_infallible(),
      }),
      (Match, Some(id)) => Ok(Self::Match {
        id: UserId::load(id).unwrap_infallible(),
      }),
      (Quiescent, None) => Ok(Self::Quiescent),
      (Welcome, None) => Ok(Self::Welcome),
      (Bio | Quiescent | Welcome, Some(payload)) => Err(Error::PromptLoadSuperfluousPayload {
        discriminant,
        payload,
      }),
      (Candidate | Match, None) => Err(Error::PromptLoadMissingPayload { discriminant }),
    }
  }
}

impl Value for PromptDiscriminant {
  type Err = Error;
  type Storage = i64;

  fn load(storage: Self::Storage) -> Result<Self, Self::Err> {
    let discriminant = u64::load(storage).unwrap_infallible();

    discriminant
      .try_into()
      .context(error::PromptLoadBadDiscriminant { discriminant })
  }

  fn store(self) -> Self::Storage {
    (self as u64).store()
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
