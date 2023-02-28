use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

type TCP = SecureTCPTransport;
type Credentials = StaticLoginCredentials;

#[tokio::main]
pub async fn init(user: Option<String>, token: Option<String>, channel: String) {
    let config = get_config(user, token);

    let (mut messages, client) = TwitchIRCClient::<TCP, Credentials>::new(config);

    let handle_messages = tokio::spawn(async move {
        while let Some(message) = messages.recv().await {
            match message {
                ServerMessage::Privmsg(privmsg) => {
                    println!("{}: {}", privmsg.sender.login, privmsg.message_text);
                }
                ServerMessage::Notice(notice) => {
                    println!("NOTICE: {}", notice.message_text);
                }
                _ => (),
            }
        }
    });

    client.join(channel).unwrap();
    handle_messages.await.unwrap();
}

fn get_config(user: Option<String>, token: Option<String>) -> ClientConfig<StaticLoginCredentials> {
    if user.is_none() || token.is_none() {
        return ClientConfig::default();
    }

    let user = user.unwrap();
    let credentials = StaticLoginCredentials::new(user, token);

    ClientConfig::new_simple(credentials)
}
