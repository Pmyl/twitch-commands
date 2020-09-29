use std::fmt::{Display, Formatter, Result};
use crate::{s};

#[derive(Debug)]
pub enum ChatEvent {
    Message(ChatMessage)
}

impl Display for ChatEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ChatEvent::Message(message) => write!(f, "{}: {} - mod: {}", message.name, message.content, s!(message.is_mod))
        }
    }
}

#[derive(Debug)]
pub struct ChatMessage {
    pub name: String,
    pub content: String,
    pub is_mod: bool
}
