use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

use crate::commands::registry::Registry;
use crate::database;
use crate::database::sqlite;
use crate::messages::Message;

type Tcp = SecureTCPTransport;
type Credentials = StaticLoginCredentials;

#[tokio::main]
pub async fn init(user: Option<String>, token: Option<String>, channel: String, prefix: String) {
    let config = get_config(user, token);

    let (mut messages, client) = TwitchIRCClient::<Tcp, Credentials>::new(config);

    let handler = client.clone();
    let cloned_channel = channel.clone();

    let handle_messages = tokio::spawn(async move {
        while let Some(message) = messages.recv().await {
            match message {
                ServerMessage::Privmsg(privmsg) => {
                    let message = Message::make(&privmsg.message_text, &privmsg.sender.login);

                    if message.has_subject() {
                        let sender = privmsg.sender.login.as_str();
                        let subject = message.subject.clone().unwrap();
                        let content = message.content.clone();

                        let response = message.get_response().await;
                        log::debug!("@{sender} triggered the subject \"{subject}\" with message: \"{content}\". Response: {response}");

                        handler
                            .say(cloned_channel.to_owned(), response)
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
                        let command = Registry::get(original_cmd);

                        if let Some(command) = command {
                            let args = args.join(" ");
                            let sender = privmsg.sender.login.as_str();

                            if command.requires_trust() && !sqlite::is_trusted(sender) {
                                log::warn!("User {sender} tried to run the \"{}\" command without permission. Consider adding them to the trusted users list.", command.name());

                                handler
                                    .say(
                                        cloned_channel.to_owned(),
                                        format!(
                                            "@{sender} you are not authorized to run this command."
                                        ),
                                    )
                                    .await
                                    .unwrap();
                            } else {
                                let response = command.execute(args.as_str(), sender).await;
                                log::debug!("@{sender} triggered builtin command \"{original_cmd}\" with args: \"{args}\". Response: {response}");

                                handler
                                    .say(cloned_channel.to_owned(), response)
                                    .await
                                    .unwrap();
                            }
                        } else {
                            let sender = privmsg.sender.login.as_str();
                            let response = database::sqlite::get_command_response(original_cmd);

                            match response {
                                Ok(response) => {
                                    log::debug!("@{sender} triggered custom command \"{original_cmd}\". Response: {response}");
                                    handler
                                        .say(cloned_channel.to_owned(), response)
                                        .await
                                        .unwrap();
                                }
                                Err(e) => {
                                    log::debug!("@{sender} triggered custom command \"{original_cmd}\". There was an error.");
                                    log::error!("Error fetching command \"{original_cmd}\": {e}");
                                }
                            }
                        }
                    }
                }
                ServerMessage::Notice(notice) => {
                    let message = notice.message_text;
                    log::info!("NOTICE: {message}");
                }
                ServerMessage::Join(join) => {
                    let (user, channel) = (join.user_login, join.channel_login);
                    log::info!("{user} joined the channel {channel}");
                }
                ServerMessage::Part(part) => {
                    let (user, channel) = (part.user_login, part.channel_login);
                    log::info!("{user} left the channel {channel}");
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
