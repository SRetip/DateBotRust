use dotenv::dotenv;

use sqlx::postgres::PgPoolOptions;
use telegram_bot::{
    callback_handler, command_handler, get_about, get_age, get_city, get_gender,
    get_interested_gender, get_interested_max_age, get_interested_min_age, get_name, Command,
    HandlerResult, State,
};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    pretty_env_logger::init();

    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(dptree::case![State::GetAge(user)].endpoint(get_age))
        .branch(dptree::case![State::GetGender(user)].endpoint(get_gender))
        .branch(dptree::case![State::GetCity(user)].endpoint(get_city))
        .branch(dptree::case![State::GetName(user)].endpoint(get_name))
        .branch(dptree::case![State::GetAbout(user)].endpoint(get_about))
        .branch(dptree::case![State::GetInterestedMinAge(user)].endpoint(get_interested_min_age))
        .branch(dptree::case![State::GetInterestedMaxAge(user)].endpoint(get_interested_max_age))
        .branch(dptree::case![State::GetInterestedGender(user)].endpoint(get_interested_gender))
        .branch(dptree::endpoint(invalid_command));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Invalid command.").await?;
    Ok(())
}
