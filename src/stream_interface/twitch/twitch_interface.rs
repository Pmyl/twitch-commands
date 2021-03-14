use twitchchat::{Connector, Dispatcher, Runner};
use futures::stream::{Stream};
use futures::*;
use tokio::stream::StreamExt;
use std::fmt::{Display, Formatter, Error};
use std::sync::Arc;
use twitchchat::messages::{Privmsg};
use twitch_api2::pubsub::channel_points::{ChannelPointsChannelV1};
use twitch_api2::pubsub::{TopicSubscribe, Response};
use websocket::{ClientBuilder, Message, OwnedMessage, WebSocketError};
use crate::stream_interface::events::{ChatEvent, ChatMessage};
use crate::{s};
use crate::utils::app_config::TwitchStreamConfig;
use websocket::message::OwnedMessage::Text;
use std::thread;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub async fn connect_to_twitch(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    info!("Connecting to stream: {}", options);
    let chat_stream = create_messages_stream(options.clone()).await;
    let channel_rewards_stream = create_channel_rewards_stream(options).await;

    // thread::spawn(move || {
    //     for message in receiver.incoming_messages() {
    //         match message.unwrap() {
    //             Text(text) => {
    //                 let result = Response::parse(&text).unwrap();
    //                 match result {
    //                     Response::Message { data } => {
    //                         info!("Found some data {:?}", data)
    //                     },
    //                     _ => debug!("response parsed but it's not a message??")
    //                 }
    //             },
    //             _ => debug!("PubSub message received but wasn't a text??")
    //         }
    //     }
    // });
    
    chat_stream //.merge(channel_rewards_stream)
}

async fn create_channel_rewards_stream(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let topic = ChannelPointsChannelV1 {
        channel_id: 1234
    };

    let command = TopicSubscribe::listen(
        &[topic],
        options.token,
        s!("????")
    ).to_command().expect("Serializing failed");

    let mut client = ClientBuilder::new("wss://pubsub-edge.twitch.tv").unwrap().connect_insecure().unwrap();
    let (mut receiver, mut sender) = client.split().unwrap();
    sender.send_message(&Message::text(command)).unwrap();

    let (tx, rx) = channel::<ChatEvent>(100);
    // 
    // tokio::spawn(async move {
    //     for message in receiver.incoming_messages() {
    //         match message.unwrap() {
    //             Text(text) => {
    //                 let result = Response::parse(&text).unwrap();
    //                 match result {
    //                     Response::Message { data } => {
    //                         info!("Found some data {:?}", data)
    //                     },
    //                     _ => debug!("response parsed but it's not a message??")
    //                 }
    //             },
    //             _ => debug!("PubSub message received but wasn't a text??")
    //         }
    //     }
    // });

    async move {
        loop {
            for message in receiver.incoming_messages() {
                match message.unwrap() {
                    Text(text) => {
                        let result = Response::parse(&text).unwrap();
                        match result {
                            Response::Message { data } => {
                                info!("Found some data {:?}", data)
                            },
                            _ => debug!("response parsed but it's not a message??")
                        }
                    },
                    _ => debug!("PubSub message received but wasn't a text??")
                }
            }
        }
    }.await;
    
    rx
    
    // stream::iter(iterator)
        // .map(|_| {
        //     ChatEvent::Message(ChatMessage {
        //         content: s!("lol"),
        //         is_mod: false,
        //         name: s!("me")
        //     })
            // match message.unwrap() {
            //     Text(text) => {
            //         let result = Response::parse(&text).unwrap();
            //         match result {
            //             Response::Message { mut data } => {
            //                 info!("Found some data {:?}", data);
            //                 ChatEvent::Message(ChatMessage {
            //                     content: s!("lol"),
            //                     is_mod: false,
            //                     name: s!("me")
            //                 })
            //             },
            //             _ => {
            //                 // debug!("response parsed but it's not a message??")
            //                 panic!("response parsed but it's not a message??")
            //             }
            //         }
            //     },
            //     _ => {
            //         // debug!("PubSub message received but wasn't a text??")
            //         panic!("PubSub message received but wasn't a text??")
            //     }
            // }
        // })
    
    // loop {
    //     tokio::select! {
    //         Some(t) = rx.recv() => t,
    //         //     let result = Response::parse(&text).unwrap();
    //         //     match result {
    //         //         Response::Message { data } => {
    //         //             info!("Found some data {:?}", data)
    //         //         },
    //         //         _ => debug!("response parsed but it's not a message??")
    //         //     }
    //         // },
    //         else => {
    //             error!("PubSub message received but wasn't a text??");
    //             break;
    //         }
    //     }
    // }
}

async fn create_messages_stream(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let TwitchConnectOptions { user, token, channel } = options;
    let dispatcher = Dispatcher::new();

    let (mut runner, mut control) = Runner::new(dispatcher.clone());

    let connector = Connector::new(move || {
        let user = user.clone();
        let token = token.clone();
        async move {
            twitchchat::native_tls::connect_easy(&user, &token).await
        }
    });

    tokio::task::spawn(async move {
        runner.run_to_completion(connector).await
    });

    debug!("waiting for irc ready");
    let ready = dispatcher
        .wait_for::<twitchchat::events::IrcReady>()
        .await
        .unwrap();
    info!("Our nickname: {}", ready.nickname);

    let mut writer = control.writer().clone();

    writer.join(channel.clone()).await.unwrap();

    info!("Joined channel: {}", channel);

    dispatcher.clear_subscriptions_all();

    map_events(dispatcher)
}

fn map_events(dispatcher: Dispatcher) -> impl Stream<Item = ChatEvent> {
    let priv_msg = dispatcher.subscribe::<twitchchat::events::Privmsg>();

    priv_msg.map(|msg: Arc<Privmsg>| {
        ChatEvent::Message(ChatMessage {
            name: s!(msg.name),
            content: s!(msg.data),
            is_mod: msg.tags.get("mod").map_or(false, |mod_tag| mod_tag == "1")
                || msg.channel == format!("#{}", msg.name) // weird hack: the owner apparently is not a mod and there is no tag to identify the ownership
        })
    })
}

#[derive(Clone)]
pub struct TwitchConnectOptions {
    pub user: String,
    pub token: String,
    pub channel: String
}

impl From<TwitchStreamConfig> for TwitchConnectOptions {
    fn from(config: TwitchStreamConfig) -> Self {
        TwitchConnectOptions { user: config.user, token: config.token, channel: config.channel }
    }
}

impl Display for TwitchConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "User: {}, Channel: {}", &self.user, &self.channel)
    }
}
