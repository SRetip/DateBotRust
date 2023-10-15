use teloxide::{types::CallbackQuery, Bot};

use crate::{
    command::go_function, get_duplet_by_id, get_user_by_chat_id, update_duplet, HandlerResult, UserReact, DupletStatus,
};

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> HandlerResult {
    if let Some(data) = q.data {
        if let Some(data) = data.split_once("_") {
            let mut duplet = get_duplet_by_id(&pool, data.0).await?;
            match data.1 {
                "like" => {
                    if duplet.first_user_react == UserReact::None {
                        duplet.first_user_react = UserReact::Like;
                    } else {
                        duplet.second_user_react = UserReact::Like;
                        duplet.status = DupletStatus::Match;
                    }
                }
                "dislike" => {
                    if duplet.first_user_react == UserReact::None {
                        duplet.first_user_react = UserReact::Dislike;
                    } else {
                        duplet.second_user_react = UserReact::Dislike;
                        duplet.status = DupletStatus::Mismatch;
                    }
                }
                _ => return Ok(()),
            }
            update_duplet(&pool, &duplet).await?;
            if let Some(message) = q.message {
                if let Some(user) = get_user_by_chat_id(&pool, message.chat.id.0).await? {
                    go_function(user, &pool, &bot, message.chat.id).await?;
                }
            }
        }
    }
    Ok(())
}
