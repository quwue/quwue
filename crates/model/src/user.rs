use crate::common::*;

#[derive(Debug, Eq, PartialEq)]
pub struct User {
  pub id:             u64,
  pub discord_id:     UserId,
  pub prompt_message: Option<PromptMessage>,
  pub welcomed:       bool,
  pub bio:            Option<String>,
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
      _ => None,
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
      Quiescent => {},
      Candidate => match content.to_lowercase().as_str() {
        "yes" => todo!(),
        "no" => todo!(),
        _ => {},
      },
      Bio =>
        return Some(Action::SetBio {
          text: content.to_string(),
        }),
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
      Quiescent => {},
      Candidate => match emoji {
        ThumbsUp => todo!(),
        ThumbsDown => todo!(),
      },
      Bio => {},
    }

    None
  }

  fn next_prompt(&self, action: Action) -> Prompt {
    use Prompt::*;

    if !(self.welcomed || action == Action::Welcome) {
      return Welcome;
    }

    if self.bio.is_none() {
      if let Action::SetBio { .. } = action {
      } else {
        return Bio;
      }
    }

    Quiescent
  }
}
