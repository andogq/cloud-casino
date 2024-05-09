CREATE TABLE bets (
    -- User that placed the bet
    user INTEGER NOT NULL,

    -- Date the bet is for
    date DATE NOT NULL,

    -- Values for the bet
    temperature FLOAT NOT NULL,
    range FLOAT NOT NULL,
    rain BOOLEAN NOT NULL,

    -- Money wagered by the user
    wager FLOAT NOT NULL,

    -- When the bet was placed or updated
    time_placed DATETIME NOT NULL,

    -- Each user can only place a bet on one date
    PRIMARY KEY (user, date)
);

CREATE TABLE forecasts (
    -- Date that the forecast is for
    date DATE NOT NULL,

    -- Date that the forecast was retrieved
    date_retrieved DATETIME NOT NULL,

    -- Whether rain is forecast for this day
    rain FLOAT NOT NULL,

    -- Minimum and maximum temperature for this day
    minimum_temperature FLOAT NOT NULL,
    maximum_temperature FLOAT NOT NULL,

    -- WMO weather code for this day
    weather_code INTEGER NOT NULL,

    -- Only one forecast per date per date retrieved
    PRIMARY KEY (date, date_retrieved)
);

CREATE TABLE historical_weather (
    -- Date that the weather was for
    date DATE NOT NULL PRIMARY KEY,

    -- Average temperature for this day
    temperature FLOAT NOT NULL,

    -- Whether it rained on this day
    rain BOOLEAN NOT NULL,

    -- Date that the historical weather was pulled
    date_retrieved DATETIME NOT NULL
);

CREATE TABLE payouts (
    -- User and date of the bet
    bet_user INTEGER NOT NULL,
    bet_date DATE NOT NULL,

    payout_date DATETIME NOT NULL,

    PRIMARY KEY (bet_user, bet_date),
    FOREIGN KEY (bet_user, bet_date) REFERENCES bets (user, date)
);
