CREATE TYPE Permission AS ENUM ( 'USER', 'ADMIN' );

CREATE TABLE "users" (
    uuid VARCHAR PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,

    permission Permission default 'user',
    tokenserial bigint default 1,

    timestamp bigint DEFAULT EXTRACT(EPOCH FROM NOW())
)
