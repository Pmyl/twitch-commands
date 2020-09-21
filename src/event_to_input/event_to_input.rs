use async_trait::async_trait;
use crate::stream_interface::events::ChatEvents;

#[async_trait]
pub trait MessageToInput {
    async fn execute(&mut self, event: ChatEvents);
}
