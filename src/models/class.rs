use crate::schema::class_infos::dsl::class_infos;
use crate::schema::class_infos::{
    audio, characteristic, cota_id, description, image, model, name, properties, symbol, video,
};
use crate::schema::token_class_audios::dsl::token_class_audios;
use crate::schema::token_class_audios::{cota_id as audio_cota_id, idx, name as audio_name, url};
use crate::utils::error::Error;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

use super::get_conn;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct ClassInfo {
    pub name:                String,
    pub symbol:              String,
    pub description:         String,
    pub image:               String,
    pub audio:               String,
    pub audios:              Vec<ClassAudio>,
    pub video:               String,
    pub model:               String,
    pub meta_characteristic: String,
    pub properties:          String,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Eq, PartialEq, Default)]
pub struct ClassInfoDb {
    pub name:           String,
    pub symbol:         String,
    pub description:    String,
    pub image:          String,
    pub audio:          String,
    pub video:          String,
    pub model:          String,
    pub characteristic: String,
    pub properties:     String,
}

pub fn get_class_info_by_cota_id(cota_id_: [u8; 20]) -> Result<Option<ClassInfo>, Error> {
    let cota_id_hex = hex::encode(cota_id_);
    let classes: Vec<ClassInfoDb> = class_infos
        .select((
            name,
            symbol,
            description,
            image,
            audio,
            video,
            model,
            characteristic,
            properties,
        ))
        .filter(cota_id.eq(cota_id_hex.clone()))
        .limit(1)
        .load::<ClassInfoDb>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query class info error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            Ok,
        )?;
    if classes.is_empty() {
        return Ok(None);
    }
    let class = classes.get(0).unwrap();
    let audios = get_class_audios_by_cota_id(cota_id_hex)?;
    let class_info = ClassInfo {
        name: class.name.clone(),
        symbol: class.symbol.clone(),
        description: class.description.clone(),
        image: class.image.clone(),
        audio: class.audio.clone(),
        audios,
        video: class.video.clone(),
        model: class.model.clone(),
        meta_characteristic: class.characteristic.clone(),
        properties: class.properties.clone(),
    };
    Ok(Some(class_info))
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Eq, PartialEq, Default)]
pub struct ClassAudio {
    pub cota_id: String,
    pub name:    String,
    pub url:     String,
    pub idx:     u32,
}

pub fn get_class_audios_by_cota_id(cota_id_hex: String) -> Result<Vec<ClassAudio>, Error> {
    let audios = token_class_audios
        .select((audio_cota_id, audio_name, url, idx))
        .filter(audio_cota_id.eq(cota_id_hex))
        .load::<ClassAudio>(&get_conn())
        .map_or_else(
            |e| {
                error!("Query class audios error: {}", e.to_string());
                Err(Error::DatabaseQueryInvalid(e.to_string()))
            },
            Ok,
        )?;
    let new_audios = audios
        .into_iter()
        .map(|audio_info| ClassAudio {
            cota_id: format!("0x{}", audio_info.cota_id),
            name:    audio_info.name,
            url:     audio_info.url,
            idx:     audio_info.idx,
        })
        .collect();
    Ok(new_audios)
}
