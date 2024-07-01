CREATE TABLE users (
    user_id BIGSERIAL PRIMARY KEY,
    oidc_id TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    admin_role TEXT NOT NULL DEFAULT 'none' CHECK (admin_role IN ('none', 'moderator', 'admin')),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE forums (
    forum_id BIGSERIAL PRIMARY KEY,
    forum_name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    is_nsfw BOOLEAN NOT NULL,
    is_banned BOOLEAN NOT NULL DEFAULT FALSE,
    tags TEXT,
    icon_url TEXT,
    banner_url TEXT,
    num_members INT NOT NULL DEFAULT 0,
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (forum_id, forum_name)
);

CREATE TABLE forum_subscriptions (
   subscription_id BIGSERIAL PRIMARY KEY,
   user_id BIGINT NOT NULL REFERENCES users (user_id),
   forum_id BIGINT NOT NULL REFERENCES forums (forum_id),
   timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
   CONSTRAINT unique_subscription UNIQUE (user_id, forum_id)
);


CREATE TABLE posts (
    post_id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    markdown_body TEXT,
    is_meta_post BOOLEAN NOT NULL DEFAULT FALSE,
    is_nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    spoiler_level INT NOT NULL DEFAULT 0,
    tags TEXT,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    meta_post_id BIGINT,
    forum_id BIGINT NOT NULL REFERENCES forums (forum_id),
    forum_name TEXT NOT NULL REFERENCES forums (forum_name),
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    creator_name TEXT NOT NULL,
    moderated_body TEXT,
    moderator_id BIGINT REFERENCES users (user_id),
    moderator_name TEXT,
    num_comments INT NOT NULL DEFAULT 0,
    score INT NOT NULL DEFAULT 0,
    score_minus INT NOT NULL DEFAULT 0,
    recommended_score REAL NOT NULL GENERATED ALWAYS AS (
            score * (2^(3 * (2 - EXTRACT(EPOCH FROM (scoring_timestamp - create_timestamp))/(3600 * 24))))
        ) STORED,
    trending_score REAL NOT NULL GENERATED ALWAYS AS (
            score * (2^(8 * (1 - EXTRACT(EPOCH FROM (scoring_timestamp - create_timestamp))/(3600 * 24))))
        ) STORED,
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edit_timestamp TIMESTAMPTZ,
    scoring_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE comments (
    comment_id BIGSERIAL PRIMARY KEY,
    body TEXT NOT NULL,
    markdown_body TEXT,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    moderated_body TEXT,
    parent_id BIGINT REFERENCES comments (comment_id),
    post_id BIGINT NOT NULL REFERENCES posts (post_id),
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    creator_name TEXT NOT NULL,
    moderator_id BIGINT REFERENCES users (user_id),
    moderator_name TEXT,
    score INT NOT NULL DEFAULT 0,
    score_minus INT NOT NULL DEFAULT 0,
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edit_timestamp TIMESTAMPTZ
);

CREATE TABLE votes (
    vote_id BIGSERIAL PRIMARY KEY,
    post_id BIGINT NOT NULL REFERENCES posts (post_id),
    comment_id BIGINT REFERENCES comments (comment_id),
    user_id BIGINT NOT NULL REFERENCES users (user_id),
    value SMALLINT NOT NULL CHECK (value IN (-1, 1)),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_vote UNIQUE NULLS NOT DISTINCT (post_id, comment_id, user_id)
);

CREATE TABLE user_forum_roles (
    role_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users (user_id),
    forum_id BIGINT NOT NULL REFERENCES forums (forum_id),
    forum_name TEXT NOT NULL REFERENCES forums (forum_name),
    permission_level TEXT NOT NULL CHECK (permission_level IN ('moderate', 'ban', 'configure', 'elect', 'lead')),
    grantor_id BIGINT NOT NULL REFERENCES users (user_id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_role UNIQUE NULLS NOT DISTINCT (user_id, forum_id)
);

-- index to guarantee maximum 1 leader per forum
CREATE UNIQUE INDEX unique_forum_leader ON user_forum_roles (forum_id, permission_level)
    WHERE (user_forum_roles.permission_level = 'lead');

CREATE TABLE user_bans (
    ban_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users (user_id),
    forum_id BIGINT,
    forum_name TEXT,
    moderator_id BIGINT NOT NULL REFERENCES users (user_id),
    until_timestamp TIMESTAMPTZ,
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT forum_exists FOREIGN KEY (forum_id, forum_name) REFERENCES forums (forum_id, forum_name)
);
