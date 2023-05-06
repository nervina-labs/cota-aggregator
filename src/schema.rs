// @generated automatically by Diesel CLI.

diesel::table! {
    check_infos (id) {
        id -> Bigint,
        check_type -> Unsigned<Tinyint>,
        block_number -> Unsigned<Bigint>,
        block_hash -> Char,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    claimed_cota_nft_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        cota_id_crc -> Unsigned<Integer>,
        token_index -> Unsigned<Integer>,
        out_point -> Char,
        out_point_crc -> Unsigned<Integer>,
        lock_hash -> Char,
        lock_hash_crc -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    class_infos (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        version -> Varchar,
        name -> Varchar,
        symbol -> Varchar,
        description -> Varchar,
        image -> Varchar,
        audio -> Varchar,
        video -> Varchar,
        model -> Varchar,
        characteristic -> Varchar,
        properties -> Varchar,
        localization -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    define_cota_nft_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        total -> Unsigned<Integer>,
        issued -> Unsigned<Integer>,
        configure -> Unsigned<Tinyint>,
        lock_hash -> Char,
        lock_hash_crc -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}


diesel::table! {
    extension_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Char,
        lock_hash_crc -> Unsigned<Integer>,
        key -> Char,
        value -> Char,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}


diesel::table! {
    hold_cota_nft_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        token_index -> Unsigned<Integer>,
        state -> Unsigned<Tinyint>,
        configure -> Unsigned<Tinyint>,
        characteristic -> Char,
        lock_hash -> Char,
        lock_hash_crc -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}


diesel::table! {
    issuer_infos (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Char,
        version -> Varchar,
        name -> Varchar,
        avatar -> Varchar,
        description -> Varchar,
        localization -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}


diesel::table! {
    joy_id_infos (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Varchar,
        version -> Varchar,
        name -> Varchar,
        avatar -> Varchar,
        description -> Varchar,
        extension -> Varchar,
        pub_key -> Char,
        credential_id -> Varchar,
        alg -> Char,
        front_end -> Varchar,
        device_name -> Varchar,
        device_type -> Varchar,
        cota_cell_id -> Char,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    register_cota_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Char,
        lock_script_id -> Bigint,
        cota_cell_id -> Unsigned<Bigint>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    schema_migrations (version) {
        version -> Bigint,
        dirty -> Bool,
    }
}

diesel::table! {
    scripts (id) {
        id -> Bigint,
        code_hash -> Char,
        code_hash_crc -> Unsigned<Integer>,
        hash_type -> Unsigned<Tinyint>,
        args -> Text,
        args_crc -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    social_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Char,
        lock_hash_crc -> Unsigned<Integer>,
        recovery_mode -> Unsigned<Tinyint>,
        must -> Unsigned<Tinyint>,
        total -> Unsigned<Tinyint>,
        signers -> Text,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}


diesel::table! {
    sub_key_infos (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Varchar,
        pub_key -> Char,
        credential_id -> Varchar,
        alg -> Char,
        front_end -> Varchar,
        device_name -> Varchar,
        device_type -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}


diesel::table! {
    sub_key_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Varchar,
        sub_type -> Char,
        ext_data -> Unsigned<Integer>,
        alg_index -> Unsigned<Integer>,
        pubkey_hash -> Char,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    token_class_audios (id) {
        id -> Bigint,
        cota_id -> Char,
        url -> Varchar,
        name -> Varchar,
        idx -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::table! {
    withdraw_cota_nft_kv_pairs (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        cota_id_crc -> Unsigned<Integer>,
        token_index -> Unsigned<Integer>,
        out_point -> Char,
        out_point_crc -> Unsigned<Integer>,
        tx_hash -> Char,
        state -> Unsigned<Tinyint>,
        configure -> Unsigned<Tinyint>,
        characteristic -> Char,
        receiver_lock_script_id -> Bigint,
        lock_hash -> Char,
        lock_hash_crc -> Unsigned<Integer>,
        lock_script_id -> Bigint,
        version -> Unsigned<Tinyint>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    check_infos,
    claimed_cota_nft_kv_pairs,
    class_infos,
    define_cota_nft_kv_pairs,
    extension_kv_pairs,
    hold_cota_nft_kv_pairs,
    issuer_infos,
    joy_id_infos,
    register_cota_kv_pairs,
    schema_migrations,
    scripts,
    social_kv_pairs,
    sub_key_infos,
    sub_key_kv_pairs,
    token_class_audios,
    withdraw_cota_nft_kv_pairs,
);
