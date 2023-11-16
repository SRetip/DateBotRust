use crate::{domain::User, State, create_or_update_user};
use teloxide::{
    dispatching::dialogue::InMemStorage, prelude::Dialogue, requests::Requester, types::Message,
    Bot,
};

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn get_age(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
) -> HandlerResult {
    if let Some(telegram_name) = msg.chat.username() {
        user.chat_id = msg.chat.id.0;
        user.telegram_name = format!("@{telegram_name}");
        match msg.text().map(|text| text.parse::<i32>()) {
            Some(Ok(age)) => {
                user.age = age;
                dialogue.update(State::GetGender(user)).await?;
                bot.send_message(msg.chat.id, "Now enter gender ('m'/'w')")
                    .await?;
            }
            _ => {
                bot.send_message(msg.chat.id, "Not a number").await?;
            }
        }
    } else {
        bot.send_message(msg.chat.id, "You need no have telegram username")
            .await?;
    }

    Ok(())
}

pub async fn get_gender(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<String>()) {
        Some(Ok(gender)) => {
            user.gender = gender;
            dialogue.update(State::GetCity(user)).await?;
            bot.send_message(msg.chat.id, "Now city").await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Not a char").await?;
        }
    }
    Ok(())
}

pub async fn get_city(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
) -> HandlerResult {
    if let Some(city) = msg.text() {
        user.city = city.into();
        dialogue.update(State::GetName(user)).await?;
        bot.send_message(msg.chat.id, "Now your name").await?;
    } else {
        bot.send_message(msg.chat.id, "Not a city").await?;
    }
    Ok(())
}

pub async fn get_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
) -> HandlerResult {
    if let Some(name) = msg.text() {
        user.name = name.into();
        dialogue.update(State::GetAbout(user)).await?;
        bot.send_message(msg.chat.id, "Now something about you")
            .await?;
    } else {
        bot.send_message(msg.chat.id, "Not a name").await?;
    }
    Ok(())
}

pub async fn get_about(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
) -> HandlerResult {
    if let Some(about) = msg.text() {
        user.about = about.into();
        dialogue.update(State::GetInterestedMinAge(user)).await?;
        bot.send_message(msg.chat.id, "Now min interested age")
            .await?;
    } else {
        bot.send_message(msg.chat.id, "Not a about").await?;
    }
    Ok(())
}

pub async fn get_interested_min_age(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(min_age)) => {
            user.interested_min_age = min_age;
            dialogue.update(State::GetInterestedMaxAge(user)).await?;
            bot.send_message(msg.chat.id, "Now max interested age")
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Not number").await?;
        }
    }
    Ok(())
}

pub async fn get_interested_max_age(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(age)) => {
            user.interested_max_age = age;
            dialogue.update(State::GetInterestedGender(user)).await?;
            bot.send_message(msg.chat.id, "Now interested gender")
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Not number").await?;
        }
    }
    Ok(())
}

pub async fn get_interested_gender(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<String>()) {
        Some(Ok(gender)) => {
            user.interested_gender = gender;
            create_or_update_user(&user, &pool).await?;
            dialogue.update(State::Ready).await?;
            bot.send_message(msg.chat.id, format!("you - {:?}", user))
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Not char").await?;
        }
    }
    Ok(())
}
