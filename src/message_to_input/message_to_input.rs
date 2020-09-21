use async_trait::async_trait;

#[async_trait]
pub trait MessageToInput {
    async fn execute(&mut self, message: String);
}