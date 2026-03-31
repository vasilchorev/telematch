CREATE TABLE IF NOT EXISTS match_notifications (
    user_id BIGINT NOT NULL,
    other_user_id BIGINT NOT NULL,
    shown_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_match_notifications
        PRIMARY KEY (user_id, other_user_id),

    CONSTRAINT fk_match_notifications_user
        FOREIGN KEY (user_id)
            REFERENCES profiles(telegram_user_id)
            ON DELETE CASCADE,

    CONSTRAINT fk_match_notifications_other_user
        FOREIGN KEY (other_user_id)
            REFERENCES profiles(telegram_user_id)
            ON DELETE CASCADE,

    CONSTRAINT chk_match_notifications_not_self
        CHECK (user_id <> other_user_id)
);
