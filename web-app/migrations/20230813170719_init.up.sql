CREATE TABLE forums (
                          id BIGSERIAL PRIMARY KEY,
                          name TEXT NOT NULL,
                          description TEXT NOT NULL,
                          nsfw BOOLEAN NOT NULL DEFAULT FALSE,
                          creator_id TEXT NOT NULL,
                          timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE contents (
                          id BIGSERIAL PRIMARY KEY,
                          title TEXT NOT NULL,
                          body TEXT NOT NULL,
                          edited BOOLEAN NOT NULL DEFAULT FALSE,
                          score INT NOT NULL DEFAULT 0,
                          parent_id BIGINT,
                          forum_id BIGINT,
                          timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);
