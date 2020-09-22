use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ChatEvents {
    Message(ChatMessage)
}

impl Display for ChatEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ChatEvents::Message(message) => write!(f, "{}: {} - mod: {}", message.name, message.content, message.is_mod.to_string())
        }
    }
}

#[derive(Debug)]
pub struct ChatMessage {
    pub name: String,
    pub content: String,
    pub is_mod: bool
}
