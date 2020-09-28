use futures::stream::{Stream};
use futures::{pin_mut, StreamExt as Ext};
use std::fmt::Display;
use tokio::sync::mpsc::{Sender};

pub trait StreamItemReceiver {
    type Item;
    type Output;
    fn receive(&mut self, item: Self::Item) -> Self::Output;
}

pub enum StreamEvent<T> {
    Item(T),
    Stop
}

pub async fn run_on_stream<T: Display, O>(items: impl Stream<Item = StreamEvent<T>>, mut item_receiver: impl StreamItemReceiver<Item = T, Output = Option<O>>, mut notifier: Sender<O>) {
    pin_mut!(items);

    loop {
        tokio::select! {
            Some(item) = items.next() => {
                match item {
                    StreamEvent::Item(item) => {
                        println!("run_on_stream::received {}", item);
                        if let Some(output) = item_receiver.receive(item) {
                            match notifier.send(output).await {
                                Ok(_) => println!("run_on_stream::send_ok"),
                                Err(e) => println!("run_on_stream::send_error::{}", e)
                            };
                        }
                    }
                    StreamEvent::Stop => {
                        println!("run_on_stream::stopped");
                        break;
                    }
                }
            }

            else => {
                eprintln!("Something bad has happened.");
                break;
            }
        }
    }
}

/**
 * `stop_on_event!(my_stream, { MyEvents:MyEvent(ref evt) => evt.is_stopping, _ => false })`
 *
 * First parameter is the stream, second parameter is a match body (without `match ev`)
 */
#[macro_export]
macro_rules! stop_on_event {
    ($stream: ident, $match_body:tt) => {{
        use crate::utils::run_on_stream::StreamEvent;

        $stream.map(|ev| {
            if match ev $match_body {
                StreamEvent::Stop
            } else {
                StreamEvent::Item(ev)
            }
        })
    }};
}
