CREATE TABLE IF NOT EXISTS issuer_infos (
    id bigint NOT NULL AUTO_INCREMENT,
    block_number bigint unsigned NOT NULL,
    lock_hash char(64) NOT NULL,
    version varchar(40) NOT NULL,
    `name` varchar(255) NOT NULL,
    avatar varchar(500) NOT NULL,
    description varchar(1000) NOT NULL,
    localization varchar(1000) NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    KEY index_issuer_infos_on_block_number (block_number),
    CONSTRAINT uc_issuer_infos_on_lock_hash UNIQUE (lock_hash)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS class_infos (
    id bigint NOT NULL AUTO_INCREMENT,
    block_number bigint unsigned NOT NULL,
    cota_id char(40) NOT NULL,
    version varchar(40) NOT NULL,
    `name` varchar(255) NOT NULL,
    symbol varchar(255) NOT NULL,
    description varchar(1000) NOT NULL,
    image varchar(500) NOT NULL,
    audio varchar(500) NOT NULL,
    video varchar(500) NOT NULL,
    model varchar(500) NOT NULL,
    characteristic varchar(1000) NOT NULL,
    properties varchar(1000) NOT NULL,
    localization varchar(1000) NOT NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    KEY index_class_infos_on_block_number (block_number),
    CONSTRAINT uc_class_infos_on_cota_id UNIQUE (cota_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS token_class_audios (
    id bigint NOT NULL AUTO_INCREMENT,
    cota_id char(40) NOT NULL,
    url varchar(255) NOT NULL DEFAULT '' COMMENT 'audio url',
    name varchar(255) NOT NULL DEFAULT '' COMMENT 'name',
    idx  int unsigned  NOT NULL DEFAULT 0  COMMENT 'idx',
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY `uc_cota_id_idx_on_class_audios` (`cota_id`,`idx`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
