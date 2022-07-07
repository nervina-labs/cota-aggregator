ALTER TABLE withdraw_cota_nft_kv_pairs
     ADD tx_hash char(64) AFTER out_point_crc,
     ADD lock_script_id bigint NOT NULL DEFAULT 3094967296 AFTER lock_hash_crc;
