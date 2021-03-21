use std::fmt::{Display, Formatter, Error};
use std::borrow::Borrow;
use url::Url;
use futures::stream::{Stream};
use futures::{SinkExt};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::{ReceiverStream};
use twitch_api2::pubsub::channel_points::{ChannelPointsChannelV1, ChannelPointsChannelV1Reply, Redemption, Reward};
use twitch_api2::pubsub::{TopicSubscribe, Response, TopicData, TwitchResponse};
use twitch_api2::types::User;
use tokio_tungstenite::{connect_async};
use tokio_tungstenite::tungstenite::{Message};
use twitch_irc::{TwitchIRCClient, TCPTransport, ClientConfig};
use twitch_irc::login::{StaticLoginCredentials};
use twitch_irc::message::ServerMessage;
use tokio::sync::mpsc::channel;
use tokio::time;
use std::time::Duration;
use tokio::sync::Mutex;
use std::sync::Arc;
use rand::{thread_rng, Rng};
use std::ops::Range;
use serde::{Serialize};
use crate::stream_interface::events::{ChatEvent, ChatMessage, ChatAction};
use crate::{s};
use crate::utils::app_config::TwitchStreamConfig;
use crate::stream_interface::twitch::user_id_from_login_name::user_id_from_login_name;

pub async fn connect_to_twitch(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    info!("Connecting to stream: {}", options);
    let chat_stream = create_messages_stream(options.clone()).await;
    let channel_rewards_stream = create_channel_rewards_stream_tungstenite(options).await;
    chat_stream.merge(channel_rewards_stream)
}

#[derive(Serialize)]
struct PingCommand {
    #[serde(rename = "type")]
    _type: String
}

impl PingCommand {
    pub fn new() -> PingCommand {
        PingCommand {
            _type: s!("PING")
        }
    }
}

async fn create_channel_rewards_stream_tungstenite(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let topic = ChannelPointsChannelV1 {
        channel_id: user_id_from_login_name(options.clone())
    };

    let command = TopicSubscribe::listen(
        &[topic],
        options.token,
        s!("????")
    ).to_command().expect("Serializing failed");

    let url = Url::parse("wss://pubsub-edge.twitch.tv").unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Can't connect to websocket");

    let (mut sender, mut receiver) = futures::StreamExt::split(ws_stream);
    sender.send(Message::Text(command.clone())).await.expect("Message sent to pubsub twitch failed");

    let ponged_original = Arc::new(Mutex::new(false));
    let ponged_check = Arc::clone(&ponged_original);
    let ponged_update = Arc::clone(&ponged_original);
    let ping_command = serde_json::to_string(&PingCommand::new()).unwrap();
    
    debug!("{}", ping_command.clone());
    debug!("{}", command.clone());

    tokio::spawn(async move {
        loop {
            let jitter = thread_rng().gen_range::<u64, Range<u64>>(Range::<u64> {
                start: 1,
                end: 10
            });
            time::sleep(Duration::from_secs(30 + jitter)).await;
            debug!("PubSub Sending Ping");
            sender.send(Message::Text(ping_command.clone())).await.expect("Not able to send Ping to websocket PubSub");

            time::sleep(Duration::from_secs(10)).await;
            let mut ponged_data = ponged_check.lock().await;
            if *ponged_data == true {
                debug!("PubSub Pong checked, received within 10 seconds");
                *ponged_data = false;
            } else {
                error!("PubSub disconnected because Pong not sent within 10 seconds of Ping, reconnection not implemented");
                break;
            }
        }
    });
    let (tx, rx) = channel::<ChatEvent>(100);

    tokio::spawn(async move {
        loop {
            let msg = receiver.next().await.unwrap();
            match msg {
                Ok(Message::Text(text)) => {
                    let result = Response::parse(&text).unwrap();
                    match result {
                        Response::Message {
                            data: TopicData::ChannelPointsChannelV1 {
                                reply, ..
                            }
                        } => {
                            if let ChannelPointsChannelV1Reply::RewardRedeemed {
                                redemption: Redemption {
                                    reward: Reward {
                                        title, ..
                                    },
                                    user: User {
                                        display_name: user_name, ..
                                    }, ..
                                }, ..
                            } = reply.borrow() {
                                info!("Redeemed {:?}!", title);
                                tx.send(ChatEvent::Action(ChatAction {
                                    name: user_name.to_owned(),
                                    action_id: "reward_redeem:".to_owned() + title
                                })).await.unwrap();
                            }
                        },
                        Response::Response(tr) => {
                            if let TwitchResponse { error: Some(ref error), .. } = tr {
                                if !error.eq("") {
                                    panic!("Connection to channel rewards failed {}", error);
                                }
                            }
                            debug!("PubSub message response parsed, unhandled twitch response {:?}", tr);
                        },
                        Response::Reconnect => {
                            debug!("PubSub message response parsed, unhandled reconnect");
                        },
                        Response::Message { .. } => {
                            debug!("PubSub message response parsed, it's a message but not the one we want");
                        },
                        Response::Pong => {
                            debug!("PubSub Pong received");
                            let mut ponged_data = ponged_update.lock().await;
                            *ponged_data = true;
                        }
                    }
                }
                _ => {
                    debug!("PubSub message response is not text {:?}", msg);
                }
            }
        }
    });

    ReceiverStream::new(rx)
}

async fn create_messages_stream(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let config = ClientConfig::new_simple(StaticLoginCredentials::new(options.user, Some(options.token)));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(config);
    
    let (tx, rx) = channel::<ChatEvent>(100);

    tokio::spawn(async move {
        let join_handle = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                if let ServerMessage::Privmsg(msg) = message {
                    debug!("Irc Private Message received {:?}", msg);
                    let has_mod_tag;
                    match msg.source.tags.0.get("mod") {
                        Some(Some(value)) => has_mod_tag = value == "1",
                        _ => has_mod_tag = false
                    };

                    let has_broadcaster_badge = msg.badges.into_iter().any(|badge| badge.name == "broadcaster");
                    tx.send(ChatEvent::Message(ChatMessage {
                        name: s!(msg.sender.name),
                        content: s!(msg.message_text),
                        is_mod: has_mod_tag || has_broadcaster_badge
                    })).await.unwrap();
                } else {
                    debug!("Irc message that is not a Private Message {:?}", message);
                }
            }
        });

        client.join("pranessa".to_owned());

        join_handle.await.unwrap();
    });
    
    ReceiverStream::new(rx)
}

#[derive(Clone)]
pub struct TwitchConnectOptions {
    pub user: String,
    pub token: String,
    pub channel: String,
    pub client_id: String
}

impl From<TwitchStreamConfig> for TwitchConnectOptions {
    fn from(config: TwitchStreamConfig) -> Self {
        TwitchConnectOptions { user: config.user, token: config.token, channel: config.channel, client_id: config.client_id }
    }
}

impl Display for TwitchConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "User: {}, Channel: {}", &self.user, &self.channel)
    }
}
