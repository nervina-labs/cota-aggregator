table! {
    check_infos (id) {
        id -> Bigint,
        check_type -> Unsigned<Tinyint>,
        block_number -> Unsigned<Bigint>,
        block_hash -> Char,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

table! {
    class_info_versions (id) {
        id -> Bigint,
        old_block_number -> Unsigned<Bigint>,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        old_version -> Varchar,
        version -> Varchar,
        old_name -> Varchar,
        name -> Varchar,
        old_symbol -> Varchar,
        symbol -> Varchar,
        old_description -> Varchar,
        description -> Varchar,
        old_image -> Varchar,
        image -> Varchar,
        old_audio -> Varchar,
        audio -> Varchar,
        old_video -> Varchar,
        video -> Varchar,
        old_model -> Varchar,
        model -> Varchar,
        old_characteristic -> Varchar,
        characteristic -> Varchar,
        old_properties -> Varchar,
        properties -> Varchar,
        old_localization -> Varchar,
        localization -> Varchar,
        action_type -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

table! {
    define_cota_nft_kv_pair_versions (id) {
        id -> Bigint,
        old_block_number -> Unsigned<Bigint>,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        total -> Unsigned<Integer>,
        old_issued -> Unsigned<Integer>,
        issued -> Unsigned<Integer>,
        configure -> Unsigned<Tinyint>,
        lock_hash -> Char,
        action_type -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

table! {
    extension_kv_pair_versions (id) {
        id -> Bigint,
        old_block_number -> Unsigned<Bigint>,
        block_number -> Unsigned<Bigint>,
        key -> Char,
        value -> Char,
        old_value -> Char,
        lock_hash -> Char,
        action_type -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

table! {
    hold_cota_nft_kv_pair_versions (id) {
        id -> Bigint,
        old_block_number -> Unsigned<Bigint>,
        block_number -> Unsigned<Bigint>,
        cota_id -> Char,
        token_index -> Unsigned<Integer>,
        old_state -> Unsigned<Tinyint>,
        state -> Unsigned<Tinyint>,
        configure -> Unsigned<Tinyint>,
        old_characteristic -> Char,
        characteristic -> Char,
        old_lock_hash -> Char,
        lock_hash -> Char,
        action_type -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

table! {
    issuer_info_versions (id) {
        id -> Bigint,
        old_block_number -> Unsigned<Bigint>,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Char,
        old_version -> Varchar,
        version -> Varchar,
        old_name -> Varchar,
        name -> Varchar,
        old_avatar -> Varchar,
        avatar -> Varchar,
        old_description -> Varchar,
        description -> Varchar,
        old_localization -> Varchar,
        localization -> Varchar,
        action_type -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

table! {
    joy_id_info_versions (id) {
        id -> Bigint,
        old_block_number -> Unsigned<Bigint>,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Varchar,
        old_version -> Varchar,
        version -> Varchar,
        old_name -> Varchar,
        name -> Varchar,
        old_avatar -> Varchar,
        avatar -> Varchar,
        old_description -> Varchar,
        description -> Varchar,
        old_extension -> Varchar,
        extension -> Varchar,
        old_nickname -> Varchar,
        nickname -> Varchar,
        pub_key -> Char,
        credential_id -> Varchar,
        alg -> Char,
        cota_cell_id -> Char,
        action_type -> Unsigned<Tinyint>,
        tx_index -> Unsigned<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    joy_id_infos (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Varchar,
        version -> Varchar,
        name -> Varchar,
        avatar -> Varchar,
        description -> Varchar,
        extension -> Varchar,
        nickname -> Varchar,
        pub_key -> Char,
        credential_id -> Varchar,
        alg -> Char,
        cota_cell_id -> Char,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

table! {
    schema_migrations (version) {
        version -> Bigint,
        dirty -> Bool,
    }
}

table! {
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

table! {
    sub_key_infos (id) {
        id -> Bigint,
        block_number -> Unsigned<Bigint>,
        lock_hash -> Varchar,
        pub_key -> Char,
        credential_id -> Varchar,
        alg -> Char,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
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

allow_tables_to_appear_in_same_query!(
    check_infos,
    claimed_cota_nft_kv_pairs,
    class_info_versions,
    class_infos,
    define_cota_nft_kv_pair_versions,
    define_cota_nft_kv_pairs,
    extension_kv_pair_versions,
    extension_kv_pairs,
    hold_cota_nft_kv_pair_versions,
    hold_cota_nft_kv_pairs,
    issuer_info_versions,
    issuer_infos,
    joy_id_info_versions,
    joy_id_infos,
    register_cota_kv_pairs,
    schema_migrations,
    scripts,
    sub_key_infos,
    withdraw_cota_nft_kv_pairs,
);
