use crate::db::{mysql::get_define_cota_of_lock_hash, types::DefineDb};
use crate::request::define::DefineReq;
use crate::smt::constants::DEFINE_NFT_SMT_TYPE;
use cota_smt::common::{Uint16, Uint32, *};
use cota_smt::define::DefineCotaNFTEntriesBuilder;
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{Blake2bHasher, H256, SMT};

fn generate_define_key(cota_id: &[u8]) -> (DefineCotaNFTId, H256) {
    let cota_id = CotaId::from_slice(cota_id).unwrap();
    let smt_type = Uint16::from_slice(&DEFINE_NFT_SMT_TYPE.to_be_bytes()).unwrap();
    let define_key = DefineCotaNFTIdBuilder::default()
        .cota_id(cota_id)
        .smt_type(smt_type)
        .build();
    let mut define_key_bytes = [0u8; 32];
    define_key_bytes[0..22].copy_from_slice(define_key.as_slice());
    let key = H256::from(define_key_bytes);
    (define_key, key)
}

fn generate_define_value(total: &[u8], issued: &[u8], configure: u8) -> (DefineCotaNFTValue, H256) {
    let define_value = DefineCotaNFTValueBuilder::default()
        .total(Uint32::from_slice(total).unwrap())
        .issued(Uint32::from_slice(issued).unwrap())
        .configure(Byte::from(configure))
        .build();
    let mut define_value_bytes = [0u8; 32];
    define_value_bytes[0..9].copy_from_slice(define_value.as_slice());
    let value = H256::from(define_value_bytes);
    (define_value, value)
}

pub fn generate_define_smt(define_req: DefineReq) -> (String, String) {
    let mut smt = SMT::default();

    let db_defines = get_define_cota_of_lock_hash(define_req.lock_hash);
    if !db_defines.is_empty() {
        for DefineDb {
            cota_id,
            total,
            issued,
            configure,
        } in db_defines
        {
            let (_, key) = generate_define_key(&cota_id);
            let (_, value) =
                generate_define_value(&total.to_be_bytes(), &issued.to_be_bytes(), configure);
            smt.update(key, value).expect("SMT update leave error");
        }
    }

    let mut update_leaves: Vec<(H256, H256)> = Vec::with_capacity(1);
    let DefineReq {
        cota_id,
        total,
        issued,
        configure,
        ..
    } = define_req;
    let (define_key, key) = generate_define_key(&cota_id);
    let (define_value, value) = generate_define_value(&total, &issued, configure);

    smt.update(key, value).expect("SMT update leave error");
    update_leaves.push((key, value));

    let root_hash = smt.root().clone();
    let mut root_hash_bytes = [0u8; 32];
    root_hash_bytes.copy_from_slice(root_hash.as_slice());
    let root_hash_hex = hex::encode(root_hash_bytes);

    println!("smt root hash: {:?}", root_hash_hex);

    let define_merkle_proof = smt
        .merkle_proof(update_leaves.iter().map(|leave| leave.0).collect())
        .unwrap();
    let define_merkle_proof_compiled = define_merkle_proof.compile(update_leaves.clone()).unwrap();
    let verify_result = define_merkle_proof_compiled
        .verify::<Blake2bHasher>(&root_hash, update_leaves.clone())
        .expect("smt proof verify failed");
    assert!(verify_result, "smt proof verify failed");

    let merkel_proof_vec: Vec<u8> = define_merkle_proof_compiled.into();
    let merkel_proof_bytes = BytesBuilder::default()
        .extend(merkel_proof_vec.iter().map(|v| Byte::from(*v)))
        .build();

    let mut action_vec: Vec<u8> = Vec::new();
    action_vec.extend("Create a new NFT collection with ".as_bytes());
    let define_total = if u32::from_be_bytes(define_req.total) == 0u32 {
        "unlimited".as_bytes()
    } else {
        &define_req.total
    };
    action_vec.extend(define_total);
    action_vec.extend(" edition".as_bytes());
    let action_bytes = BytesBuilder::default()
        .set(action_vec.iter().map(|v| Byte::from(*v)).collect())
        .build();

    let define_entries = DefineCotaNFTEntriesBuilder::default()
        .define_keys(
            DefineCotaNFTKeyVecBuilder::default()
                .set(vec![define_key])
                .build(),
        )
        .define_values(
            DefineCotaNFTValueVecBuilder::default()
                .set(vec![define_value])
                .build(),
        )
        .proof(merkel_proof_bytes)
        .action(action_bytes)
        .build();

    let define_entries_hex = hex::encode(define_entries.as_slice());

    println!("define_entries_hex: {:?}", define_entries_hex);

    (root_hash_hex, define_entries_hex)
}
