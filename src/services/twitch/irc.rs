use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

use crate::commands::BuiltinCommand;
use crate::database;
use crate::messages::Message;

type Tcp = SecureTCPTransport;
type Credentials = StaticLoginCredentials;

#[tokio::main]
pub async fn init(user: Option<String>, token: Option<String>, channel: String, prefix: String) {
    let config = get_config(user, token);

    let (mut messages, client) = TwitchIRCClient::<Tcp, Credentials>::new(config);

    let handler = client.clone();
    let clonned_channel = channel.clone();

    let handle_messages = tokio::spawn(async move {
        while let Some(message) = messages.recv().await {
            match message {
                ServerMessage::Privmsg(privmsg) => {
                    let message = Message::make(&privmsg.message_text, &privmsg.sender.login);

                    if message.has_subject() {
                        let response = message.get_response();

                        handler
                            .say(clonned_channel.to_owned(), response)
                            .await
                            .unwrap();
                    } else if privmsg.message_text.starts_with(&prefix) {
                        let mut args = privmsg
                            .message_text
                            .strip_prefix(&prefix)
                            .unwrap()
                            .split(' ')
                            .collect::<Vec<&str>>();

                        let original_cmd = args.remove(0);
                        let command = BuiltinCommand::from_string(original_cmd);

                        if let Some(command) = command {
                            let args = args.join(" ");
                            let sender = privmsg.sender.login.as_str();

                            let response = command.execute(args.as_str(), sender).await;

                            handler
                                .say(clonned_channel.to_owned(), response)
                                .await
                                .unwrap();
                        } else {
                            let response = database::sqlite::get_command_response(original_cmd);

                            if let Ok(response) = response {
                                handler
                                    .say(clonned_channel.to_owned(), response)
                                    .await
                                    .unwrap();
                            }
                        }
                    }
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
