use super::helper::SqlConnection;
use crate::schema::class_infos::dsl::class_infos;
use crate::schema::class_infos::{
    audio, cota_id, description, image, model, name, properties, schema, symbol, video,
};
use crate::utils::error::Error;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Eq, PartialEq)]
pub struct ClassInfoDb {
    pub name:        String,
    pub symbol:      String,
    pub description: String,
    pub image:       String,
    pub audio:       String,
    pub video:       String,
    pub model:       String,
    pub schema:      String,
    pub properties:  String,
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
            schema,
            properties,
        ))
        .filter(cota_id.eq(cota_id_hex))
        .load::<ClassInfoDb>(conn)
        .map_or_else(
            |e| {
                error!("Query class info error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            |classes_| Ok(classes_),
        )?;
    Ok(classes.get(0).cloned())
}
