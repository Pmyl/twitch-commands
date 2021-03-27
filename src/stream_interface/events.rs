use std::fmt::{Display, Formatter, Result};
use crate::{s};

#[derive(Debug)]
#[derive(Clone)]
pub enum ChatEvent {
    Message(ChatMessage),
    Action(ChatAction)
}

impl Display for ChatEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ChatEvent::Message(message) => write!(f, "{}: {} - mod: {}", message.name, message.content, s!(message.is_mod)),
            ChatEvent::Action(action) => write!(f, "{}: {} - {}", action.name, action.action_name, action.action_id)
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ChatMessage {
    pub name: String,
    pub content: String,
    pub is_mod: bool
}

#[derive(Clone)]
#[derive(Debug)]
pub struct ChatAction {
    pub name: String,
    pub action_id: String,
    pub action_name: String
}
