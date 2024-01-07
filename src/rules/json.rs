use std::collections::HashMap;

use super::*;
use derive_more::From;

#[derive(Debug, Clone, PartialEq, From)]
pub enum Json {
    Null,
    Number(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

declare_rules! {
    pub JsonRules {
        #[import (Whitespace) as ws]
        #[import (Float) as float]
        #[import (Boolean) as boolean]
        #[import (StringRules) as string]

        // Padded with whitespace on both sides
        start /* Json */ {
            ((ws::ws_ml) json (ws::ws_ml)) => |v| v(1);
        }

        json /* Json */ {
            ("null") => |_| Json::Null.into_value();
            ((float::float)) => |v| Json::Number(*v(0).downcast().unwrap()).into_value();
            ((boolean::boolean)) => |v| Json::Boolean(*v(0).downcast().unwrap()).into_value();
            ((string::string)) => |v| Json::String(*v(0).downcast().unwrap()).into_value();
            ("[" (ws::ws_ml) list_inner (ws::ws_ml) "]") => |v| Json::Array(*v(2).downcast().unwrap()).into_value();
            ("{" (ws::ws_ml) object_inner (ws::ws_ml) "}") => |v| Json::Object(*v(2).downcast().unwrap()).into_value();
        }

        list_inner /* Vec<Json> */ {
            () => |_| Vec::<Json>::new().into_value();
            (list_items) => |v| v(0);
        }

        list_items /* Vec<Json> */ {
            (json) => |v| vec![*v(0).downcast::<Json>().unwrap()].into_value();
            (list_items (ws::ws_ml) "," (ws::ws_ml) json) => |v| {
                let mut list = v(0).downcast::<Vec<Json>>().unwrap();

                list.push(*v(4).downcast::<Json>().unwrap());

                list
            };
        }

        object_inner /* HashMap<String, Json> */ {
            () => |_| HashMap::<String, Json>::new().into_value();
            (key_value_pairs)
        }

        key_value_pairs /* HashMap<String, Json> */ {
            (key_value_pair) => |v| {
                let mut map = HashMap::new();
                let (key, value) = *v(0).downcast::<(String, Json)>().unwrap();
                map.insert(key, value);

                map.into_value()
            };

            (key_value_pairs (ws::ws_ml) "," (ws::ws_ml) key_value_pair) => |v| {
                let mut map = v(0).downcast::<HashMap<String, Json>>().unwrap();
                let (key, value) = *v(4).downcast::<(String, Json)>().unwrap();
                map.insert(key, value);

                map
            };
        }

        key_value_pair /* (String, Json) */ {
            ((string::string) (ws::ws_ml) ":" (ws::ws_ml) json) => |v| {
                (*v(0).downcast::<String>().unwrap(), *v(4).downcast::<Json>().unwrap()).into_value()
            };
        }
    }
}
