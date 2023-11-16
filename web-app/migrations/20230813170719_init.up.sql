CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    oidc_id TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE forums (
    id BIGSERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    is_nsfw BOOLEAN NOT NULL,
    is_banned BOOLEAN NOT NULL DEFAULT FALSE,
    tags TEXT,
    icon_url TEXT,
    banner_url TEXT,
    creator_id BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    is_meta_post BOOLEAN NOT NULL DEFAULT FALSE,
    is_nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    spoiler_level INT NOT NULL DEFAULT 0,
    tags TEXT,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    moderated_body TEXT,
    meta_post_id BIGINT,
    forum_id BIGINT NOT NULL,
    creator_id BIGINT NOT NULL,
    score INT NOT NULL DEFAULT 0,
    score_minus INT NOT NULL DEFAULT 0,
    recommended_score INT NOT NULL DEFAULT 0,
    trending_score INT NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE comments (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    moderated_body TEXT,
    parent_id BIGINT NOT NULL,
    post_id BIGINT NOT NULL,
    forum_id BIGINT NOT NULL,
    creator_id BIGINT NOT NULL,
    score INT NOT NULL DEFAULT 0,
    score_minus INT NOT NULL DEFAULT 0,
    recommended_score INT NOT NULL DEFAULT 0,
    trending_score INT NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE votes (
    id BIGSERIAL PRIMARY KEY,
    creator_id BIGINT NOT NULL,
    content_id BIGINT NOT NULL,
    is_post BOOLEAN NOT NULL,
    value smallint CHECK (value IN (-1, 1)),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
