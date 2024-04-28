mod views;

use axum::{routing::get, Router};
use maud::Markup;
use time::OffsetDateTime;

use crate::Ctx;

use self::views::{bet_form::BetFormValue, forecast::Forecast};

async fn index() -> Markup {
    views::page(views::shell::render(
        183.40,
        vec![
            Forecast {
                date: OffsetDateTime::now_utc().date(),
                rain: 0.25,
                min: 16.0,
                max: 25.0,
            },
            Forecast {
                date: OffsetDateTime::now_utc().date().next_day().unwrap(),
                rain: 0.5,
                min: 18.0,
                max: 22.0,
            },
            Forecast {
                date: OffsetDateTime::now_utc()
                    .date()
                    .next_day()
                    .unwrap()
                    .next_day()
                    .unwrap(),
                rain: 0.0,
                min: 15.0,
                max: 27.0,
            },
        ],
        Some(BetFormValue {
            rain: true,
            min_temp: 19.0,
            max_temp: 20.0,
            wager: 61.50,
        }),
        1,
    ))
}

pub fn init() -> Router<Ctx> {
    Router::new().route("/", get(index))
}
