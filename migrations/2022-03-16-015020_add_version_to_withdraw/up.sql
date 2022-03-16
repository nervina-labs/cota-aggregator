ALTER TABLE withdraw_cota_nft_kv_pairs
    ADD version tinyint unsigned NOT NULL DEFAULT 0 AFTER lock_hash_crc;