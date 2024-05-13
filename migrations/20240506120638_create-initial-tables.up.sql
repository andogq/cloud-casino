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

    -- Payout amounts for the different components of the bet
    rain_payout FLOAT NOT NULL,
    temperature_payout FLOAT NOT NULL,

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

    -- Whether the rain and temperature were correct
    rain_correct BOOLEAN NOT NULL,
    temperature_correct BOOLEAN NOT NULL,

    PRIMARY KEY (bet_user, bet_date),
    FOREIGN KEY (bet_user, bet_date) REFERENCES bets (user, date)
);

CREATE TABLE states (
    -- Namespace for the state value
    namespace TEXT NOT NULL,

    -- Actual state value
    value TEXT NOT NULL,

    -- When the value was generated
    generated DATETIME NOT NULL,

    -- When the value was used
    redeemed DATETIME,

    -- Value must be unique within a namespace
    PRIMARY KEY (namespace, value)
);

CREATE TABLE users (
    -- Unique ID for the user
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    -- User's balance
    balance FLOAT NOT NULL,

    -- Last time user was logged in
    last_login DATETIME NOT NULL,

    -- Date that the user was first created
    created DATETIME NOT NULL,

    -- Unique authentication provider and identifier
    auth_provider TEXT NOT NULL,
    auth_identifier TEXT NOT NULL,

    UNIQUE (auth_provider, auth_identifier)
);
