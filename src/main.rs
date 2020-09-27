use tokio::stream::{StreamExt};
use futures::{join};
use tokio::sync::mpsc::{channel};
use crate::actions::action::Action;
use crate::stream_interface::twitch::twitch_interface::{connect_to_twitch, options_from_environment};
use crate::utils::run_on_stream::{run_on_stream};
use crate::stream_interface::events::ChatEvent;
use crate::event_to_action::configurable_event_to_action::configurable_event_to_action::{ConfigurableEventToAction, Configuration};
use crate::actions::handler::ActionHandler;

mod utils;
mod event_to_action;
mod stream_interface;
mod system_input;
mod actions;

#[tokio::main]
async fn main() {
    let twitch_event_stream = connect_to_twitch(options_from_environment()).await;
    let stoppable_twitch_event_stream = stop_on_event!(
        twitch_event_stream,
        { ChatEvent::Message(ref message) => message.is_mod && message.content.to_lowercase() == "!stop" }
    );

    let (txi, mut rxi) = channel::<Action>(100);
    let event_to_action = ConfigurableEventToAction::new(Configuration{options: vec![
        ("ups".to_string(), "up".to_string()),
        ("upsdowns".to_string(), "up_down".to_string()),
        ("find".to_string(), "find".to_string())
    ]}, txi);
    let mut action_handler = ActionHandler::default();

    let sender = run_on_stream(stoppable_twitch_event_stream, event_to_action);

    let receiver = async move {
        while let Some(a) = rxi.recv().await {
            eprintln!("Feed action {:?}", a);
            action_handler.feed(a).await;
        }
    };

    join!(sender, receiver);

    println!("end");
}

