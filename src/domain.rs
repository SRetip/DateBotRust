use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct User {
    pub chat_id: i64,
    pub age: i32,
    pub gender: String,
    pub city: String,
    pub name: String,
    pub telegram_name: String,
    pub about: String,
    pub interested_min_age: i32,
    pub interested_max_age: i32,
    pub interested_gender: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Duplet {
    pub id: String,
    pub first_user_id: i64,
    pub first_user_react: UserReact,
    pub second_user_id: i64,
    pub second_user_react: UserReact,
    pub status: DupletStatus,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PublicDuplet {
    pub partner_name: String,
    pub partner_telegram_name: String,
}

pub struct PublicUserAndDuplet {
    pub name: String,
    pub age: i32,
    pub city: String,
    pub gender: String,
    pub about: String,
    pub duplet_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum State {
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

#[derive(Debug, PartialEq, Eq, sqlx::Type, Clone, Copy)]
pub enum UserReact{
    Like,
    Dislike,
    None,
}

impl From<String> for UserReact {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Like" => UserReact::Like,
            "Dislike" => UserReact::Dislike,
            "None" => UserReact::None,
            _ => UserReact::None,
        }
    }
}



#[derive(Debug, PartialEq, Eq, sqlx::Type, Clone, Copy)]
pub enum DupletStatus {
    InProcess,
    Match, 
    Mismatch, 
    Prepared,
}

impl From<String> for DupletStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "InProcess" => DupletStatus::InProcess,
            "Match" => DupletStatus::Match,
            "Mismatch" => DupletStatus::Mismatch,
            "Prepared" => DupletStatus::Prepared,
            _ => DupletStatus::InProcess,
        }
    }
}

