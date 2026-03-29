-- /generate remove swipes and matches

-- Remove all records from the swipes table
DELETE FROM swipes;

-- Remove all records from the matches table
DELETE FROM match_notifications;

-- Optional: Resetting auto-increment counters if applicable
-- ALTER SEQUENCE swipes_id_seq RESTART WITH 1;
-- ALTER SEQUENCE matches_id_seq RESTART WITH 1;
