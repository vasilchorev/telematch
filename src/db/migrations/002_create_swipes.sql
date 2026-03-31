CREATE TABLE IF NOT EXISTS swipes (
                                      id BIGSERIAL PRIMARY KEY,
                                      from_user_id BIGINT NOT NULL,
                                      to_user_id BIGINT NOT NULL,
                                      is_like BOOLEAN NOT NULL,
                                      swiped_at TIMESTAMP NOT NULL DEFAULT NOW(),
                                      available_again_at TIMESTAMP NOT NULL,

                                      CONSTRAINT fk_swipes_from_user
                                          FOREIGN KEY (from_user_id)
                                              REFERENCES profiles(telegram_user_id)
                                              ON DELETE CASCADE,

                                      CONSTRAINT fk_swipes_to_user
                                          FOREIGN KEY (to_user_id)
                                              REFERENCES profiles(telegram_user_id)
                                              ON DELETE CASCADE,

                                      CONSTRAINT uq_swipes_from_to
                                          UNIQUE (from_user_id, to_user_id),

                                      CONSTRAINT chk_swipes_not_self
                                          CHECK (from_user_id <> to_user_id)
);