use twitchchat::{Connector, Dispatcher, Runner};
use futures::stream::{Stream};
use futures::stream::StreamExt;
use dotenv::{from_filename, var, Error as DOTENV_Error};
use std::fmt::{Display, Formatter, Error};
use crate::stream_interface::events::{ChatEvents, ChatMessage};
use std::sync::Arc;
use twitchchat::messages::Privmsg;

pub async fn connect_to_twitch(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvents> {
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

pub fn options_from_environment() -> TwitchConnectOptions {
    from_filename(".env").unwrap();
    TwitchConnectOptions { user: get_user().unwrap(), token: get_token().unwrap(), channel: get_channel().unwrap() }
}

fn map_events(dispatcher: Dispatcher) -> impl Stream<Item = ChatEvents> {
    let priv_msg = dispatcher.subscribe::<twitchchat::events::Privmsg>();

    priv_msg.map(|msg: Arc<Privmsg>| {
        ChatEvents::Message(ChatMessage {
            name: msg.name.to_string(),
            content: msg.data.to_string(),
            is_mod: msg.tags.get("mod").map_or(false, |mod_tag| mod_tag == "1")
        })
    })
}

pub struct TwitchConnectOptions {
    pub user: String,
    pub token: String,
    pub channel: String
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
