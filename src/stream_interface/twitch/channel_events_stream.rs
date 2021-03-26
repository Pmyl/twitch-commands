use std::borrow::Borrow;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use futures::{SinkExt, Stream};
use futures::stream::SplitSink;
use rand::{Rng, thread_rng};
use serde::{Serialize};
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio::time;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use twitch_api2::pubsub::{Response, TopicData, TopicSubscribe, TwitchResponse};
use twitch_api2::pubsub::channel_bits::{ChannelBitsEventsV2, ChannelBitsEventsV2Reply, BitsEventData};
use twitch_api2::pubsub::channel_points::{ChannelPointsChannelV1, ChannelPointsChannelV1Reply, Redemption, Reward};
use twitch_api2::types::User;
use url::Url;

use crate::s;
use crate::stream_interface::events::{ChatAction, ChatEvent};
use crate::stream_interface::twitch::twitch_interface::{TwitchConnectOptions};
use crate::stream_interface::twitch::user_id_from_login_name::user_id_from_login_name;

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

/**
 * Stream of ChatEvent for channel points rewards and bits
 */
pub async fn create_channel_events_stream(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let user_id = user_id_from_login_name(options.clone());

    let url = Url::parse("wss://pubsub-edge.twitch.tv").unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Can't connect to websocket");

    let (mut sender, mut receiver) = futures::StreamExt::split(ws_stream);

    subscribe_to_channel_points_topic(options.token.clone(), user_id, &mut sender).await;
    subscribe_to_bits_topic(options.token.clone(), user_id, &mut sender).await;

    let ponged_original = Arc::new(Mutex::new(false));
    let ponged_check = Arc::clone(&ponged_original);
    let ponged_update = Arc::clone(&ponged_original);
    let ping_command = serde_json::to_string(&PingCommand::new()).unwrap();
    
    debug!("{}", ping_command.clone());

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
                                    action_name: "reward_redeem".to_string(),
                                    action_id: title.to_string()
                                })).await.unwrap();
                            }
                        },
                        Response::Message {
                            data: TopicData::ChannelBitsEventsV2 {
                                reply, ..
                            }
                        } => {
                            if let ChannelBitsEventsV2Reply::BitsEvent {
                                data: BitsEventData {
                                    bits_used,
                                    user_name, ..
                                }, ..
                            } = reply.borrow() {
                                info!("Received {:?} bits!", bits_used);
                                tx.send(ChatEvent::Action(ChatAction {
                                    name: user_name.to_owned(),
                                    action_name: "bits".to_string(),
                                    action_id: bits_used.to_string()
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

async fn subscribe_to_channel_points_topic(token: String, user_id: u32, sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) {
    let channel_points_topic = ChannelPointsChannelV1 {
        channel_id: user_id
    };

    let channel_points_topic_subscription_command = TopicSubscribe::listen(
        &[channel_points_topic],
        token,
        s!("????")
    ).to_command().expect("Serializing failed");

    debug!("{}", channel_points_topic_subscription_command.clone());

    sender.send(Message::Text(channel_points_topic_subscription_command.clone())).await.expect("Message sent to pubsub twitch failed");
}


async fn subscribe_to_bits_topic(token: String, user_id: u32, sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) {
    let bits_topic = ChannelBitsEventsV2 {
        channel_id: user_id
    };

    let bits_topic_subscription_command = TopicSubscribe::listen(
        &[bits_topic],
        token,
        s!("????")
    ).to_command().expect("Serializing failed");

    debug!("{}", bits_topic_subscription_command.clone());

    sender.send(Message::Text(bits_topic_subscription_command.clone())).await.expect("Message sent to pubsub twitch failed");
}
