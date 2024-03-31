use bson::Bson;


pub fn bson_get<'a, T>(data: &'a Bson, key: T) -> Option<&'a Bson> where T: Into<Bson> {
    let key = Into::<Bson>::into(key);
    match data {
        Bson::Array(array) => {
            if let Some(idx) = key.as_i32() {
                Some(&array[idx as usize])
            } else if let Some(idx) = key.as_i64() {
                Some(&array[idx as usize])
            } else {
                None
            }
        },
        Bson::Document(doc) => {
            if let Some(str) = key.as_str() {
                doc.get(str)
            } else {
                None
            }
        },
        _ => None
    }
}

pub fn u64_to_bson(val: u64) -> Bson {
    Bson::Int64(i64::from_le_bytes(val.to_le_bytes()))
}

pub fn bson_to_u64(data: &Bson) -> Option<u64> {
    data.as_i64().map(|val| u64::from_le_bytes(val.to_le_bytes()))
}
