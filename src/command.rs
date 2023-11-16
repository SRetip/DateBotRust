use teloxide::{
    requests::Requester,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::command::BotCommands,
    Bot,
};

use crate::{
    get_needed_user, get_user_by_chat_id, get_user_matches_string, HandlerResult, MyDialogue,
    State, User,
};
use teloxide::payloads::SendMessageSetters;

use anyhow::Result;

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Settings,
    Go,
    Matches,
}

pub async fn command_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    cmd: Command,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> HandlerResult {
    let chat_id = msg.chat.id.0;
    let user = get_user_by_chat_id(&pool, chat_id).await?;
    match cmd {
        Command::Start => {
            dialogue.update(State::GetAge(User::default())).await?;
            bot.send_message(msg.chat.id, "First enter your age")
                .await?;
        }
        Command::Settings => {
            if let Some(_) = user {
                dialogue.update(State::GetAge(User::default())).await?;
                bot.send_message(msg.chat.id, "Reset registration.").await?;
                bot.send_message(msg.chat.id, "First enter your age")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Register first").await?;
            }
        }
        Command::Go => {
            if let Some(user) = user {
                go_function(user, &pool, &bot, msg.chat.id).await?;
            } else {
                bot.send_message(msg.chat.id, "Register first").await?;
            }
        }
        Command::Matches => {
            if let Some(user) = user {
                let message = get_user_matches_string(&pool, user.chat_id, msg.chat.id.0).await?;
                bot.send_message(msg.chat.id, message).await?;
            } else {
                bot.send_message(msg.chat.id, "Register first").await?;
            }
        }
    }
    Ok(())
}

pub async fn go_function(
    user: User,
    pool: &sqlx::Pool<sqlx::Postgres>,
    bot: &Bot,
    chat_id: ChatId,
) -> Result<()> {
    if let Some(u) = get_needed_user(&pool, user).await? {
        let keyboard = vec![vec![
            InlineKeyboardButton::callback("like", format!("{}_like", u.duplet_id)),
            InlineKeyboardButton::callback("dislike", format!("{}_dislike", u.duplet_id)),
        ]];
        bot.send_message(
            chat_id,
            format!(
                "name - {},\nage - {},\ncity - {},\ngender - {},\nabout - {}",
                u.name, u.age, u.city, u.gender, u.about
            ),
        )
        .reply_markup(InlineKeyboardMarkup::new(keyboard))
        .await?;
    } else {
        bot.send_message(
            chat_id,
            "There's no one suitable at the moment, try later with /go",
        )
        .await?;
    }
    Ok(())
}
