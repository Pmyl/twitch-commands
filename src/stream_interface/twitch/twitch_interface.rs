use twitchchat::{Connector, Dispatcher, Runner};
use futures::stream::{Stream};
use futures::stream::StreamExt;
use dotenv::{from_filename, var, Error as DOTENV_Error};
use std::fmt::{Display, Formatter, Error};
use std::sync::Arc;
use twitchchat::messages::Privmsg;
use crate::stream_interface::events::{ChatEvent, ChatMessage};
use crate::{s};

pub async fn connect_to_twitch(options: TwitchConnectOptions) -> impl Stream<Item =ChatEvent> {
    println!("Connecting... {}", options);
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

    eprintln!("waiting for irc ready");
    let ready = dispatcher
        .wait_for::<twitchchat::events::IrcReady>()
        .await
        .unwrap();
    eprintln!("our nickname: {}", ready.nickname);

    let mut writer = control.writer().clone();

    writer.join(channel.clone()).await.unwrap();

    eprintln!("Joined channel {}", channel);

    dispatcher.clear_subscriptions_all();

    map_events(dispatcher)
}

fn map_events(dispatcher: Dispatcher) -> impl Stream<Item =ChatEvent> {
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

pub struct TwitchConnectOptions {
    pub user: String,
    pub token: String,
    pub channel: String
}

impl TwitchConnectOptions {
    pub fn from_environment() -> TwitchConnectOptions {
        from_filename(".env").unwrap();
        TwitchConnectOptions { user: get_user().unwrap(), token: get_token().unwrap(), channel: get_channel().unwrap() }
    }
}

impl Display for TwitchConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "User: {}, Channel: {}", &self.user, &self.channel)
    }
}

fn get_user() -> Result<String, DOTENV_Error> {
    var("TWITCH_USER")
}

fn get_token() -> Result<String, DOTENV_Error> {
    var("TWITCH_TOKEN")
}

fn get_channel() -> Result<String, DOTENV_Error> {
    var("TWITCH_CHANNEL")
}
