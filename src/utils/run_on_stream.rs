use futures::stream::{Stream};
use futures::pin_mut;
use futures::stream::StreamExt;
use std::fmt::Display;
use async_trait::async_trait;

#[async_trait]
pub trait StreamItemReceiver {
    type Item;
    async fn receive(&mut self, item: Self::Item);
}

pub async fn run_on_stream<T: Display>(items: impl Stream<Item = T>, mut item_receiver: impl StreamItemReceiver<Item = T>) -> () {
    pin_mut!(items);
    loop {
        tokio::select! {
            Some(item) = items.next() => {
                println!("Received {}", item);
                item_receiver.receive(item).await;
            }

            else => {
                eprintln!("Something bad has happened.");
                break
            }
        }
    }
}
