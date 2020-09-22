use tokio::stream::StreamExt;
use crate::stream_interface::twitch::twitch_interface::{connect_to_twitch, options_from_environment};
use crate::event_to_input::test_event_to_input::test_event_to_input::TestEventToInput;
use crate::utils::run_on_stream::{run_on_stream};
use crate::stream_interface::events::ChatEvents;

mod utils;
mod event_to_input;
mod stream_interface;
mod system_input;

#[tokio::main]
async fn main() {
    let twitch_event_stream = connect_to_twitch(options_from_environment()).await;
    let stoppable_twitch_event_stream = stop_on_event!(
        twitch_event_stream,
        { ChatEvents::Message(ref message) => message.is_mod && message.content.to_lowercase() == "!stop" }
    );
    run_on_stream(stoppable_twitch_event_stream, TestEventToInput::new()).await;
    
    println!("end");
}

