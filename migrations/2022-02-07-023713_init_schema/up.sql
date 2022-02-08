START TRANSACTION;

CREATE TABLE IF NOT EXISTS define_cota_nft_kv_pairs (
    id bigint NOT NULL AUTO_INCREMENT,
    block_number bigint unsigned NOT NULL,
    cota_id char(40) NOT NULL,
    total int unsigned NOT NULL,
    issued int unsigned NOT NULL,
    configure tinyint unsigned NOT NULL,
    lock_hash char(64) NOT NULL,
    lock_hash_crc int unsigned NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    KEY index_define_on_block_number (block_number),
    KEY index_define_on_lock_hash_crc (lock_hash_crc),
    CONSTRAINT uc_define_on_cota_id UNIQUE (cota_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS hold_cota_nft_kv_pairs (
    id bigint NOT NULL AUTO_INCREMENT,
    block_number bigint unsigned NOT NULL,
    cota_id char(40) NOT NULL,
    token_index int unsigned NOT NULL,
    state tinyint unsigned NOT NULL,
    configure tinyint unsigned NOT NULL,
    characteristic char(40) NOT NULL,
    lock_hash char(64) NOT NULL,
    lock_hash_crc int unsigned NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    KEY index_hold_on_block_number (block_number),
    KEY index_hold_on_lock_hash_crc (lock_hash_crc),
    CONSTRAINT uc_hold_on_cota_id_and_token_index UNIQUE (cota_id, token_index)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS withdraw_cota_nft_kv_pairs (
    id bigint NOT NULL AUTO_INCREMENT,
    block_number bigint unsigned NOT NULL,
    cota_id char(40) NOT NULL,
    cota_id_crc int unsigned NOT NULL,
    token_index int unsigned NOT NULL,
    out_point char(72) NOT NULL,
    out_point_crc int unsigned NOT NULL,
    state tinyint unsigned NOT NULL,
    configure tinyint unsigned NOT NULL,
    characteristic char(40) NOT NULL,
    receiver_lock_script_id bigint NOT NULL,
    lock_hash char(64) NOT NULL,
    lock_hash_crc int unsigned NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    KEY index_withdraw_on_block_number (block_number),
    KEY index_withdraw_on_cota_id_crc_token_index (cota_id_crc, token_index),
    KEY index_withdraw_on_lock_script_id (receiver_lock_script_id),
    KEY index_withdraw_on_out_point_crc (out_point_crc),
    KEY index_withdraw_on_lock_hash_crc (lock_hash_crc)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS claimed_cota_nft_kv_pairs (
    id bigint NOT NULL AUTO_INCREMENT,
    block_number bigint unsigned NOT NULL,
    cota_id char(40) NOT NULL,
    cota_id_crc int unsigned NOT NULL,
    token_index int unsigned NOT NULL,
    out_point char(72) NOT NULL,
    out_point_crc int unsigned NOT NULL,
    lock_hash char(64) NOT NULL,
    lock_hash_crc int unsigned NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    KEY index_claimed_on_block_number (block_number),
    KEY index_claimed_on_cota_id_crc_token_index (cota_id_crc, token_index),
    KEY index_claimed_on_out_point_crc (out_point_crc),
    KEY index_claimed_on_lock_hash_crc (lock_hash_crc)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS scripts (
    id bigint NOT NULL AUTO_INCREMENT,
    code_hash char(64) NOT NULL,
    code_hash_crc int unsigned NOT NULL,
    hash_type tinyint unsigned NOT NULL,
    args text NOT NULL,
    args_crc int unsigned NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    key index_script_on_script_identifier (code_hash_crc, hash_type, args_crc)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

COMMIT;