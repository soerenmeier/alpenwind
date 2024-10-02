CREATE TABLE cinema_entries (
    id TEXT PRIMARY KEY,
    tmdb_id BIGINT,
    kind SMALLINT CHECK (kind IN (0, 1)),
    name TEXT NOT NULL,
    original_name TEXT,
    description TEXT,
    poster TEXT,
    background TEXT,
    rating REAL,
    duration INTEGER,
    first_publication SMALLINT,
    created_on TIMESTAMP NOT NULL,
    last_updated TIMESTAMP NOT NULL
);

CREATE INDEX idx_cinema_entries_tmdb_id ON cinema_entries (tmdb_id);

CREATE TABLE cinema_seasons (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL REFERENCES cinema_entries(id),
    season SMALLINT NOT NULL,
    name TEXT,
    original_name TEXT,
    created_on TIMESTAMP NOT NULL,
    UNIQUE (entry_id, season)
);

CREATE TABLE cinema_episodes (
    id TEXT PRIMARY KEY,
    season_id TEXT NOT NULL REFERENCES cinema_seasons(id),
    episode SMALLINT NOT NULL,
    name TEXT NOT NULL,
    original_name TEXT,
    publication_year SMALLINT,
    created_on TIMESTAMP NOT NULL,
    description TEXT,
    duration INTEGER,
    UNIQUE (season_id, episode)
);

CREATE TABLE cinema_media_files (
    id TEXT PRIMARY KEY,
    entry_id TEXT REFERENCES cinema_entries(id),
    episode_id TEXT REFERENCES cinema_episodes(id),
    name TEXT NOT NULL,
    size INTEGER NOT NULL,
    width SMALLINT,
    height SMALLINT,
    intro_time INTEGER,
    outro_time INTEGER,
    created_on TIMESTAMP NOT NULL
);

CREATE TABLE cinema_entry_genres (
    entry_id TEXT NOT NULL REFERENCES cinema_entries(id),
    genre_id INTEGER NOT NULL,
    PRIMARY KEY (entry_id, genre_id)
);

CREATE TABLE cinema_progress (
    entry_id TEXT REFERENCES cinema_entries(id),
    episode_id TEXT REFERENCES cinema_episodes(id),
    linked_id TEXT GENERATED ALWAYS AS (
        CASE
            WHEN entry_id IS NOT NULL THEN 'e' || entry_id
            WHEN episode_id IS NOT NULL THEN 's' || episode_id
            ELSE NULL
        END
    ) STORED,
    user_id TEXT NOT NULL,
    progress REAL NOT NULL,
    created_on TIMESTAMP NOT NULL,
    updated_on TIMESTAMP NOT NULL,
    last_watch TIMESTAMP,
    CHECK (
        (entry_id IS NOT NULL AND episode_id IS NULL) OR
        (entry_id IS NULL AND episode_id IS NOT NULL)
    ),
    PRIMARY KEY (user_id, linked_id)
);
