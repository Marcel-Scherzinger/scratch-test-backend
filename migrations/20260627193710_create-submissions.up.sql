CREATE TABLE submissions(
    id SERIAL PRIMARY KEY,
    createdat TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    agent VARCHAR(250),
    session VARCHAR(250),
    exercise VARCHAR(250) NOT NULL,
    program JSONB NOT NULL,
    report JSONB
);
