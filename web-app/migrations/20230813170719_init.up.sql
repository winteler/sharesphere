CREATE TABLE contents (
                          id BIGSERIAL PRIMARY KEY,
                          title TEXT NOT NULL,
                          body TEXT NOT NULL,
                          edited BOOLEAN NOT NULL DEFAULT FALSE,
                          score INT NOT NULL DEFAULT 0,
                          timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
                          parent_id BIGINT
);
