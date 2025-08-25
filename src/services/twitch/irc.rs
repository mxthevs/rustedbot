use std::sync::Arc;

use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

use crate::commands::registry::Registry;
use crate::commands::Command;
use crate::database;
use crate::database::sqlite;
use crate::messages::Message;

type Tcp = SecureTCPTransport;
type Credentials = StaticLoginCredentials;
type Client = TwitchIRCClient<Tcp, Credentials>;
type TokioMessage = tokio::sync::mpsc::UnboundedReceiver<ServerMessage>;

#[tokio::main]
pub async fn init(user: Option<String>, token: Option<String>, channel: String, prefix: String) {
    let config = get_config(user, token);
    let (incoming_messages, client) = Client::new(config);

    client.join(channel.clone()).unwrap();

    let handler_task = tokio::spawn(message_handler_loop(
        incoming_messages,
        client,
        channel,
        prefix,
    ));

    handler_task.await.unwrap();
}

async fn message_handler_loop(
    mut messages: TokioMessage,
    client: Client,
    channel: String,
    prefix: String,
) {
    while let Some(message) = messages.recv().await {
        let client_clone = client.clone();
        let channel_clone = channel.clone();
        let prefix_clone = prefix.clone();

        tokio::spawn(async move {
            handle_server_message(message, client_clone, channel_clone, prefix_clone).await;
        });
    }
}

async fn handle_server_message(
    message: ServerMessage,
    client: Client,
    channel: String,
    prefix: String,
) {
    match message {
        ServerMessage::Privmsg(privmsg) => {
            handle_private_message(privmsg, client, channel, prefix).await;
        }
        ServerMessage::Notice(notice) => {
            log::info!("NOTICE: {}", notice.message_text);
        }
        ServerMessage::Join(join) => {
            log::info!(
                "{} joined the channel {}",
                join.user_login,
                join.channel_login
            );
        }
        ServerMessage::Part(part) => {
            log::info!(
                "{} left the channel {}",
                part.user_login,
                part.channel_login
            );
        }
        _ => (),
    }
}

async fn handle_private_message(
    privmsg: PrivmsgMessage,
    client: Client,
    channel: String,
    prefix: String,
) {
    let sender = &privmsg.sender.login;
    let message_text = &privmsg.message_text;

    if message_text.starts_with(&prefix) {
        handle_command(message_text, sender, client, channel, &prefix).await;
    } else {
        let message = Message::make(message_text, sender);
        if message.has_subject() {
            handle_subject_message(message, sender, client, channel).await;
        }
    }
}

async fn handle_command(
    message_text: &str,
    sender: &str,
    client: Client,
    channel: String,
    prefix: &str,
) {
    let mut args = message_text.strip_prefix(prefix).unwrap().split(' ');
    let command_name = args.next().unwrap_or("");
    let command_args = args.collect::<Vec<&str>>().join(" ");

    if let Some(command) = Registry::get(command_name) {
        handle_builtin_command(command, &command_args, sender, client, channel).await;
    } else {
        handle_custom_command(command_name, sender, client, channel).await;
    }
}

async fn handle_subject_message(message: Message, sender: &str, client: Client, channel: String) {
    let subject = message.subject.clone().unwrap();
    let content = message.content.clone();
    let response = message.get_response().await;

    log::debug!("@{sender} triggered the subject `{subject}` with message `{content}`. Response: {response}");
    say(client, channel, response).await;
}

async fn handle_builtin_command(
    command: Arc<dyn Command + Send + Sync>,
    args: &str,
    sender: &str,
    client: Client,
    channel: String,
) {
    let command_name = command.name();
    if command.requires_trust() && !sqlite::is_trusted(sender) {
        log::warn!("User {sender} tried to run the `{command_name}` command without permission. Consider adding them to the trusted users list.");
        let response = format!("@{sender} you are not authorized to run this command.");
        say(client, channel, response).await;
    } else {
        let response = command.execute(args, sender).await;
        log::debug!("@{sender} triggered builtin command `{command_name}` with args `{args}`. Response: {response}");
        say(client, channel, response).await;
    }
}

async fn handle_custom_command(command_name: &str, sender: &str, client: Client, channel: String) {
    match database::sqlite::get_command_response(command_name) {
        Ok(response) => {
            log::debug!(
                "@{sender} triggered custom command `{command_name}`. Response: {response}"
            );
            say(client, channel, response).await;
        }
        Err(e) => {
            log::debug!("@{sender} triggered custom command `{command_name}`. There was an error.");
            log::error!("Error fetching command `{command_name}`: {e}");
        }
    }
}

async fn say(client: Client, channel: String, text: String) {
    if let Err(e) = client.say(channel, text).await {
        log::error!("Failed to send message: {e}");
    }
}

fn get_config(user: Option<String>, token: Option<String>) -> ClientConfig<Credentials> {
    match (user, token) {
        (Some(user), Some(token)) => {
            let credentials = StaticLoginCredentials::new(user, Some(token));
            ClientConfig::new_simple(credentials)
        }
        _ => ClientConfig::default(),
    }
}
