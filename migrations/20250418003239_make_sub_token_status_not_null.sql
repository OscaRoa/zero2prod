-- Add migration script here
BEGIN;
    UPDATE subscription_tokens
    SET status = 'unconfirmed'
    WHERE status IS NULL;
    ALTER TABLE subscription_tokens ALTER COLUMN status SET NOT NULL;
COMMIT;