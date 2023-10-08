use dotenv::dotenv;

use sqlx::{postgres::PgPoolOptions, types::Uuid};
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

use serde::{Deserialize, Serialize};

use anyhow::Result;

use lazy_format::prelude::*;

use joinery::JoinableIterator;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct User {
    chat_id: i64,
    age: i32,
    gender: String,
    city: String,
    name: String,
    telegram_name: String,
    about: String,
    interested_min_age: i32,
    interested_max_age: i32,
    interested_gender: String,
}

struct Duplet {
    id: String,
    first_user_id: i64,
    first_user_react: String, //like,dislike,in_process,
    second_user_id: i64,
    second_user_react: String, //like,dislike,in_process,
    status: String,            //in_process, match, mismatch, prepared,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct PublicDuplet {
    partner_name: String,
    partner_telegram_name: String,
}

struct PublicUserAndDuplet {
    name: String,
    age: i32,
    city: String,
    gender: String,
    about: String,
    duplet_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
enum State {
    GetAge(User),
    GetGender(User),
    GetCity(User),
    GetName(User),
    GetAbout(User),
    GetInterestedMinAge(User),
    GetInterestedMaxAge(User),
    GetInterestedGender(User),
    Ready,
}

impl Default for State {
    fn default() -> Self {
        State::GetAge(User::default())
    }
}

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Settings,
    Go,
    Matches,
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:testdb@localhost/postgres")
        .await?;
    pretty_env_logger::init();

    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        // .branch(dptree::entry().filter_command::<Command>().endpoint(command_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(dptree::case![State::GetAge(user)].endpoint(get_age))
        .branch(dptree::case![State::GetGender(user)].endpoint(get_gender))
        .branch(dptree::case![State::GetCity(user)].endpoint(get_city))
        .branch(dptree::case![State::GetName(user)].endpoint(get_name))
        .branch(dptree::case![State::GetAbout(user)].endpoint(get_about))
        .branch(dptree::case![State::GetInterestedMinAge(user)].endpoint(get_interested_min_age))
        .branch(dptree::case![State::GetInterestedMaxAge(user)].endpoint(get_interested_max_age))
        .branch(dptree::case![State::GetInterestedGender(user)].endpoint(get_interested_gender))
        .branch(
            dptree::case![State::Ready]
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(command_handler),
                )
                .branch(dptree::endpoint(invalid_command)),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn get_age(bot: Bot, dialogue: MyDialogue, msg: Message, mut user: User) -> HandlerResult {
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

async fn get_gender(bot: Bot, dialogue: MyDialogue, msg: Message, mut user: User) -> HandlerResult {
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

async fn get_city(bot: Bot, dialogue: MyDialogue, msg: Message, mut user: User) -> HandlerResult {
    if let Some(city) = msg.text() {
        user.city = city.into();
        dialogue.update(State::GetName(user)).await?;
        bot.send_message(msg.chat.id, "Now your name").await?;
    } else {
        bot.send_message(msg.chat.id, "Not a city").await?;
    }
    Ok(())
}

async fn get_name(bot: Bot, dialogue: MyDialogue, msg: Message, mut user: User) -> HandlerResult {
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

async fn get_about(bot: Bot, dialogue: MyDialogue, msg: Message, mut user: User) -> HandlerResult {
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

async fn get_interested_min_age(
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

async fn get_interested_max_age(
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

async fn get_interested_gender(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut user: User,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<String>()) {
        Some(Ok(gender)) => {
            user.interested_gender = gender;
            sqlx::query!(
                "INSERT INTO Users (chat_id, age, gender, city, name, telegram_name, about, interested_min_age, interested_max_age, interested_gender) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                user.chat_id,
                user.age,
                user.gender,
                user.city,
                user.name,
                user.telegram_name,
                user.about,
                user.interested_min_age,
                user.interested_max_age,
                user.interested_gender,
            )
            .execute(&pool)
            .await?;
            dialogue.update(State::Ready).await?;
            // save to storrage
            bot.send_message(msg.chat.id, format!("you - {:?}", user))
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Not char").await?;
        }
    }
    Ok(())
}

async fn command_handler(
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
            dialogue.update(State::GetAge(User::default())).await?;
            bot.send_message(msg.chat.id, "Reset registration.").await?;
            bot.send_message(msg.chat.id, "First enter your age")
                .await?;
        }
        Command::Go => {
            if let Some(user) = user {
                go_function(user, &pool, &bot, msg.chat.id).await?;
            } else {
                bot.send_message(msg.chat.id, "Register first").await?;
            }
        }
        Command::Matches => {
            struct InnerDuplet {
                first_user_chat_id: i64,
                first_user_name: String,
                first_user_telegram_name: String,
                second_user_name: String,
                second_user_telegram_name: String,
            }

            if let Some(user) = user {
                let matches: Vec<InnerDuplet> = sqlx::query_as!(
                    InnerDuplet,
                    r#"
                SELECT
                u1.chat_id AS first_user_chat_id,
                u1.name AS first_user_name,
                u1.telegram_name AS first_user_telegram_name,
                u2.name AS second_user_name,
                u2.telegram_name AS second_user_telegram_name
                FROM
                Duplets AS d
                LEFT JOIN
                Users AS u1 ON d.first_user_id = u1.chat_id
                LEFT JOIN
                Users AS u2 ON d.second_user_id = u2.chat_id
                WHERE
                (d.first_user_id = $1 OR d.second_user_id = $2) AND d.status = 'match';
                "#,
                    user.chat_id,
                    user.chat_id
                )
                .fetch_all(&pool)
                .await?;
                let duplets = matches
                    .iter()
                    .map(|d| {
                        if d.first_user_chat_id == msg.chat.id.0 {
                            return PublicDuplet {
                                partner_name: d.second_user_name.clone(),
                                partner_telegram_name: d.second_user_telegram_name.clone(),
                            };
                        } else {
                            return PublicDuplet {
                                partner_name: d.first_user_name.clone(),
                                partner_telegram_name: d.first_user_telegram_name.clone(),
                            };
                        }
                    })
                    .map(|d| lazy_format!("{} - {}", &d.partner_name, &d.partner_telegram_name))
                    .join_with(", ");
                bot.send_message(msg.chat.id, duplets.to_string()).await?;
            } else {
                bot.send_message(msg.chat.id, "Register first").await?;
            }
        }
    }
    Ok(())
}

async fn invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Invalid command.").await?;
    Ok(())
}

async fn get_needed_user(
    pool: &sqlx::Pool<sqlx::Postgres>,
    u: User,
) -> Result<Option<PublicUserAndDuplet>> {
    let duplet_and_user: Option<PublicUserAndDuplet> = sqlx::query_as!(
        PublicUserAndDuplet,
        r#"
    SELECT
    d.id as duplet_id,
    u1.age as age,
    u1.name as name,
    u1.city as city,
    u1.gender as gender,
    u1.about as about
    FROM
    Duplets AS d
    INNER JOIN
    Users AS u1 ON d.first_user_id = u1.chat_id
    WHERE
    d.second_user_id = $1 AND d.second_user_react = 'like';
    "#,
        u.chat_id
    )
    .fetch_optional(pool)
    .await?;
    if duplet_and_user.is_some() {
        return Ok(duplet_and_user);
    } else {
        let user: Option<User> = sqlx::query_as!(
            User,
            r#"
        SELECT *
        FROM Users u
        WHERE (select d.id
        FROM Duplets d
        WHERE (d.first_user_id = $1 AND d.second_user_id = u.chat_id)
        OR (d.first_user_id = u.chat_id AND d.second_user_id = $2)) IS NULL
        AND (u.age between $3 AND $4
        AND u.city = $5
        AND u.gender = $6
        AND $7 between u.interested_min_age AND u.interested_max_age
        AND u.interested_gender = $8)
        LIMIT 1;"#,
            u.chat_id,
            u.chat_id,
            u.interested_min_age,
            u.interested_max_age,
            u.city,
            u.interested_gender,
            u.age,
            u.gender
        )
        .fetch_optional(pool)
        .await?;
        if let Some(user) = user {
            let duplet_id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO Duplets VALUES ($1, $2, $3, $4, $5, $6);",
                duplet_id,
                u.chat_id,
                "".into(),
                user.chat_id,
                "".into(),
                "in_progress".into()
            )
            .execute(pool)
            .await?;
            return Ok(Some(PublicUserAndDuplet {
                name: user.name,
                age: user.age,
                city: user.city,
                gender: user.gender,
                about: user.about,
                duplet_id,
            }));
        } else {
            Ok(None)
        }
    }
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> HandlerResult {
    if let Some(data) = q.data {
        if let Some(data) = data.split_once("_") {
            let mut duplet =
                sqlx::query_as!(Duplet, r#"SELECT * FROM Duplets where id = $1"#, data.0)
                    .fetch_one(&pool)
                    .await?;
            match data.1 {
                "like" => {
                    if duplet.first_user_react == "" {
                        duplet.first_user_react = "like".into();
                    } else {
                        duplet.second_user_react = "like".into();
                        duplet.status = "match".into();
                    }
                }
                "dislike" => {
                    if duplet.first_user_react == "" {
                        duplet.first_user_react = "dislike".into();
                    } else {
                        duplet.second_user_react = "dislike".into();
                        duplet.status = "mismatch".into();
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

async fn go_function(
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

async fn get_user_by_chat_id(
    pool: &sqlx::Pool<sqlx::Postgres>,
    chat_id: i64,
) -> Result<Option<User>> {
    let user = sqlx::query_as!(User, r#"SELECT * FROM Users WHERE chat_id = $1"#, chat_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

async fn update_duplet(
    pool: &sqlx::Pool<sqlx::Postgres>,
    updated_duplet: &Duplet,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE Duplets SET
        first_user_id = $2,
        first_user_react = $3,
        second_user_id = $4,
        second_user_react = $5,
        status = $6
        WHERE id = $1",
        updated_duplet.id,
        updated_duplet.first_user_id,
        updated_duplet.first_user_react,
        updated_duplet.second_user_id,
        updated_duplet.second_user_react,
        updated_duplet.status,
    )
    .execute(pool)
    .await?;

    Ok(())
}