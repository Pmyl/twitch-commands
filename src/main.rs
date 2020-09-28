use tokio::stream::{StreamExt};
use futures::future::{join_all, join3};
use futures::{join};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{delay_for, Duration};
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::borrow::BorrowMut;
use crate::actions::action::{Action, ActionCategory};
use crate::stream_interface::twitch::twitch_interface::{connect_to_twitch, TwitchConnectOptions};
use crate::utils::run_on_stream::{run_on_stream};
use crate::stream_interface::events::ChatEvent;
use crate::event_to_action::configurable_event_to_action::configurable_event_to_action::{ConfigurableEventToAction, Configuration};
use crate::actions::handler::ActionHandler;
use crate::event_to_action::event_to_action::EventToAction;

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

    let mut event_to_action = ConfigurableEventToAction::new(Configuration{options: vec![
        ("ups".to_string(), "up".to_string()),
        ("upsdowns".to_string(), "up_down".to_string()),
        ("find".to_string(), "find".to_string()),
        ("find_atomic".to_string(), "find_atomic".to_string())
    ]});
    let custom_categories = event_to_action.custom_categories();

    let (category_notifier, mut category_receiver) = channel::<ActionCategory>(100);
    let stream_to_event_to_action = run_on_stream(stoppable_twitch_event_stream, event_to_action, category_notifier);

    let (mut action_queues_notifiers, mut action_queues_receivers) = build_category_queues(custom_categories);

    let sender = async move {
        while let Some(category) = category_receiver.recv().await {
            match category {
                ActionCategory::Uncategorized(item) => {
                    match action_queues_notifiers.get_mut("_uncategorized") {
                        Some(sender) => {
                            match sender.send(item).await {
                                Ok(_) => println!("category_dispatcher::with_category::send_ok"),
                                Err(e) => println!("category_dispatcher::with_category::send_error::{}", e)
                            };
                        },
                        None => {
                            // TODO: handle this error
                            println!("Some weird stuff happened");
                        }
                    }
                }
                ActionCategory::WithCategory(name, item) => {
                    match action_queues_notifiers.get_mut(&name) {
                        Some(sender) => {
                            match sender.send(item).await {
                                Ok(_) => println!("category_dispatcher::with_category::send_ok"),
                                Err(e) => println!("category_dispatcher::with_category::send_error::{}", e)
                            };
                        },
                        None => {
                            // TODO: handle this error
                            println!("Some weird stuff happened");
                        }
                    }
                }
            }
        }
    };

    let mut futures_to_run = Vec::new();

    for receiver_queue in action_queues_receivers.iter_mut() {
        let channel = actions_channel(receiver_queue.1.borrow_mut());
        futures_to_run.push(channel);
    }

    let final_future = async move { join_all(futures_to_run).await; };

    join3(stream_to_event_to_action, sender, final_future).await;

    println!("end");
}

fn build_category_queues(custom_categories: Vec<String>) -> (HashMap<String, Sender<Action>, RandomState>, HashMap<String, Receiver<Action>, RandomState>) {
    let mut notifiers_hash_map = HashMap::with_capacity(custom_categories.len());
    let mut receivers_hash_map = HashMap::with_capacity(custom_categories.len());

    let queue = channel::<Action>(100);
    notifiers_hash_map.insert("_uncategorized".to_string(), queue.0);
    receivers_hash_map.insert("_uncategorized".to_string(), queue.1);

    for category_name in custom_categories {
        let queue = channel::<Action>(100);
        notifiers_hash_map.insert(category_name.clone(), queue.0);
        receivers_hash_map.insert(category_name, queue.1);
    }

    (notifiers_hash_map, receivers_hash_map)
}

async fn actions_channel(rxi: &mut Receiver<Action>) -> () {
    let mut action_handler = ActionHandler::default();
    let actions_to_enqueue = Arc::new(Mutex::new(Vec::<Action>::new()));
    let actions_to_dequeue = actions_to_enqueue.clone();

    let receiver = async move {
        while let Some(a) = rxi.recv().await {
            eprintln!("Feed action {:?}", a);
            let mut actions = actions_to_enqueue.lock().unwrap();
            actions.push(a);
            drop(actions);
        }
    };

    let executer = async move {
        loop {
            if is_mouse_pressed_bool() {
                delay_for(Duration::from_millis(100)).await;
                continue;
            }

            let mut actions = actions_to_dequeue.lock().unwrap();
            action_handler.run(actions.deref_mut());
            drop(actions);
            delay_for(Duration::from_millis(10)).await;
        }
    };

    join!(receiver, executer);
}

