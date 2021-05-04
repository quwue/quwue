use crate::common::*;

#[derive(Debug, Eq, PartialEq)]
pub struct User {
  pub id:                u64,
  pub discord_id:        UserId,
  pub prompt_message:    Option<PromptMessage>,
  pub welcomed:          bool,
  pub bio:               Option<String>,
  pub profile_image_url: Option<Url>,
}

impl User {
  pub fn update(&self, response: &Response) -> Update {
    let prompt = if let Some(prompt_message) = self.prompt_message {
      prompt_message.prompt
    } else {
      return Update {
        prompt: Prompt::Welcome,
        action: None,
      };
    };

    let action = match response {
      Response::Message(content) => Self::action_for_message(prompt, content),
      Response::Reaction(emoji) => Self::action_for_reaction(prompt, *emoji),
      Response::Image(url) => Self::action_for_image(prompt, url.clone()),
      Response::UnrecognizedReaction(..) | Response::Custom(..) => None,
    };

    let action = if let Some(action) = action {
      action
    } else {
      return Update {
        action: None,
        prompt,
      };
    };

    Update {
      action: Some(action.clone()),
      prompt: self.next_prompt(action),
    }
  }

  fn action_for_message(prompt: Prompt, content: &str) -> Option<Action> {
    use Prompt::*;

    let content = content.trim();

    match prompt {
      Welcome =>
        if content.to_lowercase() == "ok" {
          return Some(Action::Welcome);
        },
      Candidate { .. } => match content.to_lowercase().as_str() {
        "yes" => todo!(),
        "no" => todo!(),
        _ => {},
      },
      Bio =>
        return Some(Action::SetBio {
          text: content.to_string(),
        }),
      Quiescent | ProfileImage => {},
    }

    None
  }

  fn action_for_reaction(prompt: Prompt, emoji: Emoji) -> Option<Action> {
    use Emoji::*;
    use Prompt::*;

    match prompt {
      Welcome =>
        if emoji == ThumbsUp {
          return Some(Action::Welcome);
        },
      Candidate { .. } => match emoji {
        ThumbsUp => todo!(),
        ThumbsDown => todo!(),
      },
      Quiescent | Bio | ProfileImage => {},
    }

    None
  }

  fn action_for_image(prompt: Prompt, url: Url) -> Option<Action> {
    use Prompt::*;

    match prompt {
      ProfileImage => Some(Action::SetProfileImage { url }),
      Welcome | Quiescent | Candidate { .. } | Bio => None,
    }
  }

  fn next_prompt(&self, action: Action) -> Prompt {
    if !(self.welcomed || action == Action::Welcome) {
      return Prompt::Welcome;
    }

    if self.bio.is_none() {
      if let Action::SetBio { .. } = action {
      } else {
        return Prompt::Bio;
      }
    }

    if self.profile_image_url.is_none() {
      if let Action::SetProfileImage { .. } = action {
      } else {
        return Prompt::ProfileImage;
      }
    }

    Prompt::Quiescent
  }
}
