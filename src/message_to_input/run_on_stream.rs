use futures::stream::{Stream};
use futures::pin_mut;
use futures::stream::StreamExt;
use crate::stream_interface::events::{ChatEvents};
use crate::message_to_input::message_to_input::MessageToInput;

pub async fn run_on_stream(messages: impl Stream<Item = ChatEvents>, mut message_to_input: impl MessageToInput) -> () {
    pin_mut!(messages);
    loop {
        tokio::select! {
            Some(ChatEvents::Message(message)) = messages.next() => {
                eprintln!("Received {}: {}", message.name, message.content);
                message_to_input.execute(message.content).await;
            }

            else => {
                eprintln!("ERROR");
                break
            }
        }
    }
}