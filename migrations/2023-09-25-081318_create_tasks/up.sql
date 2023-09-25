-- noinspection SqlDialectInspectionForFile

-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TYPE  valid_events AS ENUM ('started', 'ended');

CREATE TABLE IF NOT EXISTS tasks
(
    id        SERIAL  PRIMARY KEY,
    name      VARCHAR NOT NULL,
    description VARCHAR,
    timezone VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS actions
(
    id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name      VARCHAR NOT NULL,
    description VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    task_id INTEGER NOT NULL ,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS task_events
(
    id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name      VARCHAR NOT NULL,
    description VARCHAR,
    event_type valid_events NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE
);