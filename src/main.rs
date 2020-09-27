use tokio::stream::{StreamExt};
use futures::future::{join3};
use tokio::sync::mpsc::{channel};
use tokio::time::{delay_for, Duration};
use crate::actions::action::Action;
use crate::stream_interface::twitch::twitch_interface::{connect_to_twitch, TwitchConnectOptions};
use crate::utils::run_on_stream::{run_on_stream};
use crate::stream_interface::events::ChatEvent;
use crate::event_to_action::configurable_event_to_action::configurable_event_to_action::{ConfigurableEventToAction, Configuration};
use crate::actions::handler::ActionHandler;
use std::sync::{Arc, Mutex};

extern crate libc;

extern {
    fn is_mouse_pressed() -> libc::c_int;
}

fn is_mouse_pressed_bool() -> bool {
    unsafe { is_mouse_pressed() == 1 }
}

mod utils;
mod event_to_action;
mod stream_interface;
mod system_input;
mod actions;

#[tokio::main]
async fn main() {
    let twitch_event_stream = connect_to_twitch(TwitchConnectOptions::from_environment()).await;
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
    let mutex_handler_to_feed = Arc::new(Mutex::new(action_handler));
    let mutex_handler_to_run = mutex_handler_to_feed.clone();

    let sender = run_on_stream(stoppable_twitch_event_stream, event_to_action);

    let receiver = async move {
        while let Some(a) = rxi.recv().await {
            eprintln!("Feed action {:?}", a);
            let mut handler_to_feed = mutex_handler_to_feed.lock().unwrap();
            handler_to_feed.feed(a);
            drop(handler_to_feed);
        }
    };

    let executer = async move {
        loop {
            if is_mouse_pressed_bool() {
                delay_for(Duration::from_millis(100)).await;
                continue;
            }

            let mut handler_to_run = mutex_handler_to_run.lock().unwrap();
            handler_to_run.run().await;
            drop(handler_to_run);
            delay_for(Duration::from_millis(100)).await;
        }
    };

    join3(sender, receiver, executer).await;

    println!("end");
}

