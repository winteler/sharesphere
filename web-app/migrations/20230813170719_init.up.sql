CREATE TABLE users (
    user_id BIGSERIAL PRIMARY KEY,
    oidc_id TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    admin_role TEXT NOT NULL DEFAULT 'None' CHECK (admin_role IN ('None', 'Moderator', 'Admin')),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (user_id, username)
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
    user_id BIGINT NOT NULL,
    username TEXT NOT NULL,
    forum_id BIGINT NOT NULL,
    forum_name TEXT NOT NULL,
    permission_level TEXT NOT NULL CHECK (permission_level IN ('None', 'Moderate', 'Ban', 'Manage', 'Lead')),
    grantor_id BIGINT NOT NULL REFERENCES users (user_id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_role UNIQUE (user_id, forum_id),
    CONSTRAINT valid_user FOREIGN KEY (user_id, username) REFERENCES users (user_id, username),
    CONSTRAINT valid_forum FOREIGN KEY (forum_id, forum_name) REFERENCES forums (forum_id, forum_name)
);

-- index to guarantee maximum 1 leader per forum
CREATE UNIQUE INDEX unique_forum_leader ON user_forum_roles (forum_id, permission_level)
    WHERE user_forum_roles.permission_level = 'Lead';

CREATE TABLE rules (
    rule_id BIGSERIAL PRIMARY KEY,
    forum_id BIGINT,
    forum_name TEXT,
    priority SMALLINT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users (user_id),
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delete_timestamp TIMESTAMPTZ,
    CONSTRAINT valid_forum FOREIGN KEY (forum_id, forum_name) REFERENCES forums (forum_id, forum_name)
);

-- index to guarantee numbering of rules is unique
CREATE UNIQUE INDEX unique_rule ON rules (forum_id, priority)
    WHERE rules.delete_timestamp IS NULL;


CREATE TABLE user_bans (
    ban_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    username TEXT NOT NULL,
    forum_id BIGINT,
    forum_name TEXT,
    moderator_id BIGINT NOT NULL REFERENCES users (user_id),
    until_timestamp TIMESTAMPTZ,
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_user FOREIGN KEY (user_id, username) REFERENCES users (user_id, username),
    CONSTRAINT valid_forum FOREIGN KEY (forum_id, forum_name) REFERENCES forums (forum_id, forum_name)
);
