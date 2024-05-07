CREATE TABLE bets (
    -- User that placed the bet
    user INTEGER,

    -- Date the bet is for
    date DATE,

    -- Values for the bet
    temperature FLOAT,
    range FLOAT,
    rain BOOLEAN,

    -- Money wagered by the user
    wager FLOAT,

    -- When the bet was placed or updated
    time_placed DATETIME,

    -- Each user can only place a bet on one date
    PRIMARY KEY (user, date)
);

