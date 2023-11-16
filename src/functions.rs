use crate::{Duplet, PublicDuplet, PublicUserAndDuplet, User, UserReact, DupletStatus};
use anyhow::{Context, Result};
use sqlx::types::Uuid;

use lazy_format::prelude::*;

use joinery::JoinableIterator;

pub async fn get_needed_user(
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
            sqlx::query(
                "INSERT INTO Duplets VALUES ($1, $2, $3, $4, $5, $6);")
                .bind(duplet_id.clone())
                .bind(u.chat_id)
                .bind(UserReact::None)
                .bind(user.chat_id)
                .bind(UserReact::None)
                .bind(DupletStatus::InProcess)
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

pub async fn get_user_by_chat_id(
    pool: &sqlx::Pool<sqlx::Postgres>,
    chat_id: i64,
) -> Result<Option<User>> {
    let user = sqlx::query_as!(User, r#"SELECT * FROM Users WHERE chat_id = $1"#, chat_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn update_duplet(
    pool: &sqlx::Pool<sqlx::Postgres>,
    updated_duplet: &Duplet,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE Duplets SET
        first_user_id = $2,
        first_user_react = $3,
        second_user_id = $4,
        second_user_react = $5,
        status = $6
        WHERE id = $1")
        .bind(updated_duplet.id.clone())
        .bind(updated_duplet.first_user_id)
        .bind(updated_duplet.first_user_react)
        .bind(updated_duplet.second_user_id)
        .bind(updated_duplet.second_user_react)
        .bind(updated_duplet.status)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_duplet_by_id(pool: &sqlx::Pool<sqlx::Postgres>, id: &str) -> Result<Duplet> {
    sqlx::query_as!(Duplet, r#"SELECT * FROM Duplets where id = $1"#, id)
        .fetch_one(pool)
        .await
        .context("Fail to fetch duplet")
}

pub async fn get_user_matches_string(
    pool: &sqlx::Pool<sqlx::Postgres>,
    chat_id: i64,
    message_id: i64,
) -> Result<String> {
    struct InnerDuplet {
        first_user_chat_id: i64,
        first_user_name: String,
        first_user_telegram_name: String,
        second_user_name: String,
        second_user_telegram_name: String,
    }
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
        chat_id,
        chat_id
    )
    .fetch_all(pool)
    .await?;
    let duplets = matches
        .iter()
        .map(|d| {
            if d.first_user_chat_id == message_id {
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
    Ok(duplets.to_string())
}

pub async fn create_or_update_user(user: &User, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let query = format!(
        "INSERT INTO users (chat_id, age, gender, city, name, telegram_name, about, interested_min_age, interested_max_age, interested_gender)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (chat_id) DO UPDATE
         SET age = $2, gender = $3, city = $4, name = $5, telegram_name = $6, about = $7, interested_min_age = $8, interested_max_age = $9, interested_gender = $10",
    );

    sqlx::query(&query)
        .bind(&user.chat_id)
        .bind(&user.age)
        .bind(&user.gender)
        .bind(&user.city)
        .bind(&user.name)
        .bind(&user.telegram_name)
        .bind(&user.about)
        .bind(&user.interested_min_age)
        .bind(&user.interested_max_age)
        .bind(&user.interested_gender)
        .execute(pool)
        .await?;

    Ok(())
}