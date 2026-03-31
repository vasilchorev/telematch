CREATE TABLE IF NOT EXISTS profiles (
                                        id BIGSERIAL PRIMARY KEY,
                                        telegram_user_id BIGINT NOT NULL UNIQUE,
                                        name TEXT NOT NULL,
                                        gender TEXT NOT NULL,
                                        looking_for TEXT NOT NULL,
                                        age SMALLINT NOT NULL,
                                        location TEXT NOT NULL,
                                        description TEXT NOT NULL,
                                        photo_file_id TEXT NOT NULL,
                                        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );