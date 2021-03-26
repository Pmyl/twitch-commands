use tokio_stream::{StreamExt};
use futures::future::{join_all, join3};
use tokio::sync::mpsc::{channel};
use std::borrow::BorrowMut;
use simplelog::{SimpleLogger, LevelFilter, Config, WriteLogger, CombinedLogger, SharedLogger};
use std::fs::File;
use chrono::Local;
#[macro_use] extern crate log;
use crate::actions::action::{ActionCategory};
use crate::stream_interface::twitch::twitch_interface::{connect_to_twitch};
use crate::utils::run_on_stream::{run_on_stream};
use crate::stream_interface::events::ChatEvent;
use crate::event_to_action::configurable_event_to_action::configurable_event_to_action::{ConfigurableEventToAction};
use crate::event_to_action::event_to_action::EventToAction;
use crate::utils::app_config::{app_config, AppConfig};
use crate::actions::queue::{action_queue_coordinators, redirect_action_in_queue, actions_queue};

mod utils;
mod event_to_action;
mod stream_interface;
mod system_input;
mod actions;

#[tokio::main]
async fn main() {
    let configuration = app_config();
    init_logger(&configuration);

    let twitch_event_stream = connect_to_twitch(configuration.twitch_stream.into()).await;
    let stoppable_twitch_event_stream = stop_on_event!(
        twitch_event_stream,
        { ChatEvent::Message(ref message) => message.is_mod && message.content.to_lowercase() == "!stop", _ => false }
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

    info!("End of execution");
}

fn init_logger(configuration: &AppConfig) {
    let file_log_level = get_log_level(configuration.file_log_level.clone());
    let terminal_log_level = get_log_level(configuration.terminal_log_level.clone());

    let mut loggers = Vec::<Box<dyn SharedLogger>>::new();
    if terminal_log_level != LevelFilter::Off {
        println!("Logging in terminal with log level {}", terminal_log_level);
        loggers.insert(0, SimpleLogger::new(terminal_log_level, Config::default()))
    } else {
        println!("Not logging in terminal");
    }
    
    if file_log_level != LevelFilter::Off {
        let log_file_name = format!("twitch-commands_{}.log", Local::now().format("%Y%m%d%H%M%S"));
        println!("Logging in file {} with log level {}", log_file_name, file_log_level);
        loggers.insert(0, WriteLogger::new(file_log_level, Config::default(), File::create(log_file_name).unwrap()))
    } else {
        println!("Not logging in file");
    }
    

    if let Err(_) = CombinedLogger::init(loggers) {
        eprintln!("Failed initializing logger for the application, nothing will be logged.");
    }
}

fn get_log_level(log_level: String) -> LevelFilter {
    match log_level.as_str() {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warning" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info
    }
}
