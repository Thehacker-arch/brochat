-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    avatar_url TEXT DEFAULT '/images/default-avatar.png'
);

-- Create messages table
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender TEXT NOT NULL,
    target_username TEXT, -- NULL for public messages
    message_type TEXT NOT NULL CHECK (message_type IN ('chat', 'dm')),
    message TEXT NOT NULL,
    upload_url TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
