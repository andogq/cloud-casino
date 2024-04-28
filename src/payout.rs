use time::OffsetDateTime;

use crate::user::User;

pub fn count_ready(user: &User) -> usize {
    let now = OffsetDateTime::now_utc().date();

    user.data
        .outstanding_bets
        .iter()
        .filter(|day| day < &&now)
        .count()
}
