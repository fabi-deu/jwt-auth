CREATE TYPE Permission AS ENUM ( 'USER', 'ADMIN' );

CREATE TABLE "users" (
    uuid VARCHAR PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,

    permission Permission default 'USER',
    tokenid VARCHAR NOT NULL,

    timestamp bigint DEFAULT EXTRACT(EPOCH FROM NOW())
)