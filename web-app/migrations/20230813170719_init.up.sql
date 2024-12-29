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

CREATE TABLE spheres (
    sphere_id BIGSERIAL PRIMARY KEY,
    sphere_name TEXT UNIQUE NOT NULL,
    normalized_sphere_name TEXT UNIQUE NOT NULL GENERATED ALWAYS AS (
            REPLACE(LOWER(sphere_name), '-', '_')
        ) STORED,
    description TEXT NOT NULL,
    is_nsfw BOOLEAN NOT NULL,
    is_banned BOOLEAN NOT NULL DEFAULT FALSE,
    icon_url TEXT,
    banner_url TEXT,
    num_members INT NOT NULL DEFAULT 0,
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (sphere_id, sphere_name)
);

CREATE TABLE satellites (
    satellite_id BIGSERIAL PRIMARY KEY,
    satellite_name TEXT NOT NULL,
    sphere_id BIGINT NOT NULL,
    sphere_name TEXT NOT NULL,
    body TEXT NOT NULL,
    markdown_body TEXT,
    is_nsfw BOOLEAN NOT NULL,
    is_spoiler BOOLEAN NOT NULL,
    num_posts INT NOT NULL DEFAULT 0,
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    disable_timestamp TIMESTAMPTZ,
    CONSTRAINT unique_satellite_name UNIQUE (satellite_name, sphere_id),
    CONSTRAINT unique_sphere UNIQUE (sphere_id, satellite_id),
    CONSTRAINT valid_forum FOREIGN KEY (sphere_id, sphere_name) REFERENCES spheres (sphere_id, sphere_name) MATCH FULL
);

-- index to guarantee unique satellite names per forum for active satellites
CREATE UNIQUE INDEX unique_satellite ON satellites (satellite_name, sphere_id)
    WHERE satellites.disable_timestamp IS NULL;

CREATE TABLE user_sphere_roles (
    role_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    username TEXT NOT NULL,
    sphere_id BIGINT NOT NULL,
    sphere_name TEXT NOT NULL,
    permission_level TEXT NOT NULL CHECK (permission_level IN ('None', 'Moderate', 'Ban', 'Manage', 'Lead')),
    grantor_id BIGINT NOT NULL REFERENCES users (user_id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_role UNIQUE (user_id, sphere_id),
    CONSTRAINT valid_user FOREIGN KEY (user_id, username) REFERENCES users (user_id, username) MATCH FULL,
    CONSTRAINT valid_sphere FOREIGN KEY (sphere_id, sphere_name) REFERENCES spheres (sphere_id, sphere_name) MATCH FULL
);

-- index to guarantee maximum 1 leader per sphere
CREATE UNIQUE INDEX unique_sphere_leader ON user_sphere_roles (sphere_id, permission_level)
    WHERE user_sphere_roles.permission_level = 'Lead';

CREATE TABLE rules (
    rule_id BIGSERIAL PRIMARY KEY,
    rule_key BIGSERIAL, -- business id to track rule across updates
    sphere_id BIGINT,
    sphere_name TEXT,
    priority SMALLINT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users (user_id),
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delete_timestamp TIMESTAMPTZ,
    CONSTRAINT valid_sphere FOREIGN KEY (sphere_id, sphere_name) REFERENCES spheres (sphere_id, sphere_name) MATCH FULL
);

-- index to guarantee numbering of rules is unique
CREATE UNIQUE INDEX unique_rule ON rules (sphere_id, priority)
    WHERE rules.delete_timestamp IS NULL;
-- index to guarantee there is only one active rule for a given rule_key
CREATE UNIQUE INDEX unique_rule_key ON rules (rule_key)
    WHERE rules.delete_timestamp IS NULL;

CREATE TABLE sphere_categories (
    category_id BIGSERIAL PRIMARY KEY,
    sphere_id BIGINT NOT NULL,
    sphere_name TEXT NOT NULL,
    category_name TEXT NOT NULL,
    category_color SMALLINT NOT NULL,
    description TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delete_timestamp TIMESTAMPTZ,
    CONSTRAINT sphere_category UNIQUE (category_id, sphere_id),
    CONSTRAINT unique_category UNIQUE (sphere_id, category_name),
    CONSTRAINT valid_sphere FOREIGN KEY (sphere_id, sphere_name) REFERENCES spheres (sphere_id, sphere_name) MATCH FULL
);

CREATE INDEX category_order ON sphere_categories (sphere_name, is_active, category_name);

CREATE TABLE sphere_subscriptions (
   subscription_id BIGSERIAL PRIMARY KEY,
   user_id BIGINT NOT NULL REFERENCES users (user_id),
   sphere_id BIGINT NOT NULL REFERENCES spheres (sphere_id),
   timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
   CONSTRAINT unique_subscription UNIQUE (user_id, sphere_id)
);

CREATE TABLE posts (
    post_id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    markdown_body TEXT,
    link TEXT,
    link_type SMALLINT NOT NULL CHECK (link_type IN (-1, 0, 1, 2)),
    is_nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    is_spoiler BOOLEAN NOT NULL DEFAULT FALSE,
    category_id BIGINT,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    sphere_id BIGINT NOT NULL,
    sphere_name TEXT NOT NULL,
    satellite_id BIGINT REFERENCES satellites (satellite_id),
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    creator_name TEXT NOT NULL,
    is_creator_moderator BOOLEAN NOT NULL,
    moderator_message TEXT,
    infringed_rule_id BIGINT REFERENCES rules (rule_id),
    infringed_rule_title TEXT,
    moderator_id BIGINT REFERENCES users (user_id),
    moderator_name TEXT,
    num_comments INT NOT NULL DEFAULT 0,
    is_pinned BOOLEAN NOT NULL,
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
    scoring_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_sphere FOREIGN KEY (sphere_id, sphere_name) REFERENCES spheres (sphere_id, sphere_name) MATCH FULL,
    CONSTRAINT valid_satellite FOREIGN KEY (sphere_id, satellite_id) REFERENCES satellites (sphere_id, satellite_id),
    CONSTRAINT valid_sphere_category FOREIGN KEY (category_id, sphere_id) REFERENCES sphere_categories (category_id, sphere_id) MATCH SIMPLE
);

CREATE TABLE comments (
    comment_id BIGSERIAL PRIMARY KEY,
    body TEXT NOT NULL,
    markdown_body TEXT,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    moderator_message TEXT,
    infringed_rule_id BIGINT REFERENCES rules (rule_id),
    infringed_rule_title TEXT,
    parent_id BIGINT REFERENCES comments (comment_id),
    post_id BIGINT NOT NULL REFERENCES posts (post_id),
    creator_id BIGINT NOT NULL REFERENCES users (user_id),
    creator_name TEXT NOT NULL,
    is_creator_moderator BOOLEAN NOT NULL,
    moderator_id BIGINT REFERENCES users (user_id),
    moderator_name TEXT,
    is_pinned BOOLEAN NOT NULL,
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

CREATE TABLE user_bans (
    ban_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    username TEXT NOT NULL,
    sphere_id BIGINT,
    sphere_name TEXT,
    post_id BIGINT NOT NULL,
    comment_id BIGINT,
    infringed_rule_id BIGINT NOT NULL REFERENCES rules (rule_id),
    moderator_id BIGINT NOT NULL REFERENCES users (user_id),
    until_timestamp TIMESTAMPTZ,
    create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_user FOREIGN KEY (user_id, username) REFERENCES users (user_id, username) MATCH FULL,
    CONSTRAINT valid_sphere FOREIGN KEY (sphere_id, sphere_name) REFERENCES spheres (sphere_id, sphere_name) MATCH FULL
);
