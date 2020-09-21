use crate::stream_interface::twitch::twitch_interface::{connect_to_twitch, options_from_environment};
use crate::message_to_input::test_message_to_input::test_message_to_input::TestMessageToInput;
use crate::message_to_input::run_on_stream::run_on_stream;

mod message_to_input;
mod stream_interface;
mod input_controller;

#[tokio::main]
async fn main() {
    let twitch = connect_to_twitch(options_from_environment()).await;
    run_on_stream(twitch, TestMessageToInput::new()).await;
    
    // find out how to write unit tests (doc test???), write example for TestMessageToInput, in preparation to write it for ConfiguredMessageToInput
    // write unit tests for run_on_stream to learn how to write unit tests with Streams
    // implement ConfiguredMessageToInput passing a configuration Dict<MessageMatcher, ControlInput> (ControlInput e' un enum)
    // write unit tests for ConfiguredMessageToInput
    // ????
    // PROFIT!
    
    println!("end");
}
