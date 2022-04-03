use crate::models::helper::SqlConnection;
use crate::schema::class_infos::dsl::class_infos;
use crate::schema::class_infos::{
    audio, characteristic, cota_id, description, image, model, name, properties, symbol, video,
};
use crate::utils::error::Error;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Eq, PartialEq)]
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

pub fn get_class_info_by_cota_id(
    conn: &SqlConnection,
    cota_id_: [u8; 20],
) -> Result<Option<ClassInfoDb>, Error> {
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
        .filter(cota_id.eq(cota_id_hex))
        .limit(1)
        .load::<ClassInfoDb>(conn)
        .map_or_else(
            |e| {
                error!("Query class info error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            Ok,
        )?;
    Ok(classes.get(0).cloned())
}
