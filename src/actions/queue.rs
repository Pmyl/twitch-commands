use futures::{join};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;
use std::collections::HashMap;
use crate::actions::action::{Action, ActionCategory};
use crate::actions::handler::ActionHandler;
use crate::{s};

const UNCATEGORIZED_CHANNEL_NAME: &str = "_uncategorized";

pub fn action_queue_coordinators(custom_categories: Vec<String>) -> (HashMap<String, Sender<Action>>, HashMap<String, Receiver<Action>>) {
    let all_categories = add_uncategorized(custom_categories);

    let mut notifiers_hash_map = HashMap::with_capacity(all_categories.len());
    let mut receivers_hash_map = HashMap::with_capacity(all_categories.len());

    for category_name in all_categories {
        let queue = channel::<Action>(100);
        notifiers_hash_map.insert(category_name.clone(), queue.0);
        receivers_hash_map.insert(category_name, queue.1);
    }

    (notifiers_hash_map, receivers_hash_map)
}

pub async fn redirect_action_in_queue(category_receiver: &mut Receiver<ActionCategory>, queue_notifiers: &mut HashMap<String, Sender<Action>>) {
    while let Some(category) = category_receiver.recv().await {
        let category_name;
        let action;

        match category {
            ActionCategory::Uncategorized(item) => {
                category_name = s!(UNCATEGORIZED_CHANNEL_NAME);
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
                    Ok(_) => debug!("Redirect OK on category {}", category_name),
                    Err(e) => error!("redirect_action_in_queue::redirect_error::{}", e)
                };
            },
            None => {
                error!("redirect_action_in_queue::error::`received unhandled category {}`", category_name);
            }
        }
    }
}

pub async fn actions_queue(rxi: &mut Receiver<Action>) -> () {
    let mut action_handler = ActionHandler::default();
    let actions_to_enqueue = Arc::new(Mutex::new(Vec::<Action>::new()));
    let actions_to_dequeue = actions_to_enqueue.clone();

    let feeder = async move {
        while let Some(a) = rxi.recv().await {
            debug!("Feed action {:?}", a);
            let mut actions = actions_to_enqueue.lock().unwrap();
            actions.push(a);
            drop(actions);
        }
    };

    let runner = async move {
        loop {
            if !action_handler.can_handle() {
                sleep(Duration::from_millis(100)).await;
                continue;
            }

            let mut actions = actions_to_dequeue.lock().unwrap();
            action_handler.run(actions.deref_mut());
            drop(actions);
            sleep(Duration::from_millis(10)).await;
        }
    };

    join!(feeder, runner);
}

fn add_uncategorized(custom_categories: Vec<String>) -> Vec<String> {
    let mut all_categories = custom_categories.clone();
    all_categories = all_categories
        .iter()
        .map(|name| if name == UNCATEGORIZED_CHANNEL_NAME { format!("custom{}", name) } else { name.clone() })
        .collect();
    all_categories.append(vec![s!(UNCATEGORIZED_CHANNEL_NAME)].as_mut());

    all_categories
}
