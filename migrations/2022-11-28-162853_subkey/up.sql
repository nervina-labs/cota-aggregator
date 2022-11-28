CREATE TABLE IF NOT EXISTS sub_key_kv_pairs (
    id bigint NOT NULL AUTO_INCREMENT,
    block_number bigint UNSIGNED NOT NULL,
    lock_hash varchar(64) NOT NULL,
    sub_type char(6) NOT NULL,
    ext_data int UNSIGNED NOT NULL,
    alg_index int UNSIGNED NOT NULL,
    pubkey_hash char(40) NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    KEY index_sub_key_kv_pairs_on_lock_hash (lock_hash),
    KEY index_sub_key_kv_pairs_on_block_number (block_number),
    UNIQUE KEY `uc_sub_key_kv_pairs_on_lock_hash_ext_data` (`lock_hash`,`ext_data`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

