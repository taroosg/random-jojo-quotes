CREATE TABLE IF NOT EXISTS quotes (
id SERIAL PRIMARY KEY NOT NULL,
quote VARCHAR NOT NULL,
speaker VARCHAR NOT NULL,
source VARCHAR NOT NULL,
);