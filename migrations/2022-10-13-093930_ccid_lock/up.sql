ALTER TABLE register_cota_kv_pairs
    ADD cota_cell_id bigint unsigned NOT NULL DEFAULT 18446744073709551615 AFTER lock_hash,
    ADD lock_script_id bigint NOT NULL DEFAULT 3094967296 AFTER lock_hash;
    
