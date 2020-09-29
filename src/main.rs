use tokio::stream::{StreamExt};
use futures::future::{join_all, join3};
use tokio::sync::mpsc::{channel};
use std::borrow::BorrowMut;
use crate::actions::action::{ActionCategory};
use crate::stream_interface::twitch::twitch_interface::{connect_to_twitch};
use crate::utils::run_on_stream::{run_on_stream};
use crate::stream_interface::events::ChatEvent;
use crate::event_to_action::configurable_event_to_action::configurable_event_to_action::{ConfigurableEventToAction};
use crate::event_to_action::event_to_action::EventToAction;
use crate::utils::app_config::app_config;
use crate::actions::queue::{action_queue_coordinators, redirect_action_in_queue, actions_queue};

mod utils;
mod event_to_action;
mod stream_interface;
mod system_input;
mod actions;

#[tokio::main]
async fn main() {
    let configuration = app_config();
    let twitch_event_stream = connect_to_twitch(configuration.twitch_stream.into()).await;
    let stoppable_twitch_event_stream = stop_on_event!(
        twitch_event_stream,
        { ChatEvent::Message(ref message) => message.is_mod && message.content.to_lowercase() == "!stop" }
    );

    let mut event_to_action = ConfigurableEventToAction::new(configuration.mapping.into());
    let custom_categories = event_to_action.custom_categories();

    let (category_notifier, mut category_receiver) = channel::<ActionCategory>(100);
    let stream_to_event_to_action = run_on_stream(stoppable_twitch_event_stream, event_to_action, category_notifier);

    let (mut queue_notifiers, mut queue_receivers) = action_queue_coordinators(custom_categories);
    let action_in_queues_notifier = redirect_action_in_queue(&mut category_receiver, &mut queue_notifiers);
    let actions_runner_queues = queue_receivers.iter_mut().map(|qr| actions_queue(qr.1.borrow_mut()));
    let actions_runners = async move { join_all(actions_runner_queues).await; };

    join3(stream_to_event_to_action, action_in_queues_notifier, actions_runners).await;

    println!("end");
}
