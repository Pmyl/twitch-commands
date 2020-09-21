use async_trait::async_trait;
use crate::stream_interface::events::ChatEvents;

#[async_trait]
pub trait EventToInput {
    async fn execute(&mut self, event: ChatEvents);
}
