-- Add migration script here
ALTER TABLE subscription_tokens ADD COLUMN status TEXT null default 'unconfirmed';