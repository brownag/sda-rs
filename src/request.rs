use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

#[derive(Debug)]
pub struct QueryRequest {
    pub query: Option<String>,
    pub format: Option<String>,
}

impl<'de> Deserialize<'de> for QueryRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RequestVisitor;
        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = QueryRequest;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an SDA query request object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let (mut query, mut format) = (None, None);
                while let Some((key, value)) = map.next_entry::<String, String>()? {
                    match key.to_ascii_lowercase().as_str() {
                        "query" => query = Some(value),
                        "format" => format = Some(value),
                        _ => {}
                    }
                }
                Ok(QueryRequest { query, format })
            }
        }
        deserializer.deserialize_map(RequestVisitor)
    }
}

pub fn parse_form(body: &[u8]) -> Result<QueryRequest, std::string::FromUtf8Error> {
    let mut request = QueryRequest {
        query: None,
        format: None,
    };
    for pair in body.split(|byte| *byte == b'&') {
        let separator = pair
            .iter()
            .position(|byte| *byte == b'=')
            .unwrap_or(pair.len());
        let (key, value) = (&pair[..separator], pair.get(separator + 1..).unwrap_or(&[]));
        let key = decode_form_component(key)?;
        let value = decode_form_component(value)?;
        if key.eq_ignore_ascii_case("query") {
            request.query = Some(value);
        } else if key.eq_ignore_ascii_case("format") {
            request.format = Some(value);
        }
    }
    Ok(request)
}

fn decode_form_component(input: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    let mut decoded = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        match input[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < input.len() => {
                let hex = |byte: u8| match byte {
                    b'0'..=b'9' => Some(byte - b'0'),
                    b'a'..=b'f' => Some(byte - b'a' + 10),
                    b'A'..=b'F' => Some(byte - b'A' + 10),
                    _ => None,
                };
                if let (Some(high), Some(low)) = (hex(input[index + 1]), hex(input[index + 2])) {
                    decoded.push(high * 16 + low);
                    index += 3;
                } else {
                    decoded.push(input[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded)
}
