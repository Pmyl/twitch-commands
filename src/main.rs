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
use crate::event_to_action::configurable_event_to_action::configurable_event_to_action::{ConfigurableEventToAction};
use crate::actions::handler::ActionHandler;
use crate::event_to_action::event_to_action::EventToAction;
use crate::utils::app_config::app_config;

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
    let configuration = app_config();
    let twitch_event_stream = connect_to_twitch(TwitchConnectOptions::from_environment()).await;
    let stoppable_twitch_event_stream = stop_on_event!(
        twitch_event_stream,
        { ChatEvent::Message(ref message) => message.is_mod && message.content.to_lowercase() == "!stop" }
    );

    let mut event_to_action = ConfigurableEventToAction::new(configuration.mapping.into());
    let custom_categories = event_to_action.custom_categories();

    let (category_notifier, mut category_receiver) = channel::<ActionCategory>(100);
    let stream_to_event_to_action = run_on_stream(stoppable_twitch_event_stream, event_to_action, category_notifier);

    let (mut queue_notifiers, mut queue_receivers) = build_category_queues(custom_categories);

    let action_in_queues_notifier = build_action_in_queues_notifier(&mut category_receiver, &mut queue_notifiers);
    let actions_runner_queues = queue_receivers.iter_mut().map(|qr| actions_queue(qr.1.borrow_mut()));
    let actions_runners = async move { join_all(actions_runner_queues).await; };

    join3(stream_to_event_to_action, action_in_queues_notifier, actions_runners).await;

    println!("end");
}

async fn build_action_in_queues_notifier(category_receiver: &mut Receiver<ActionCategory>, queue_notifiers: &mut HashMap<String, Sender<Action>>) {
    while let Some(category) = category_receiver.recv().await {
        let category_name;
        let action;

        match category {
            ActionCategory::Uncategorized(item) => {
                category_name = String::from("_uncategorized");
                action = item;
            }
            ActionCategory::WithCategory(name, item) => {
                category_name = name.clone();
                action = item;
            }
        }

        match queue_notifiers.get_mut(&category_name) {
            Some(sender) => {
                match sender.send(action).await {
                    Ok(_) => println!("action_in_queues_notifier::send_ok::category::{}", category_name),
                    Err(e) => println!("action_in_queues_notifier::send_error::{}", e)
                };
            },
            None => {
                println!("action_in_queues_notifier::error::`received unhandled category {}`", category_name);
            }
        }
    }
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

async fn actions_queue(rxi: &mut Receiver<Action>) -> () {
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

