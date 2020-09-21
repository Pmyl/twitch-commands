#[derive(Debug)]
pub enum ChatEvents {
    Message(ChatMessage)
}

#[derive(Debug)]
pub struct ChatMessage {
    pub name: String,
    pub content: String
}