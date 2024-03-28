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
