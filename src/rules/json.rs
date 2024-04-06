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
            ((ws::ws_ml) json (ws::ws_ml)) => |v, _| v(1);
        }

        json /* Json */ {
            ("null") => |_, _| Json::Null.into_value();
            ((float::float)) => |v, _| Json::Number(*v(0).downcast().unwrap()).into_value();
            ((boolean::boolean)) => |v, _| Json::Boolean(*v(0).downcast().unwrap()).into_value();
            ((string::string)) => |v, _| Json::String(*v(0).downcast().unwrap()).into_value();
            ("[" (ws::ws_ml) list_inner (ws::ws_ml) "]") => |v, _| Json::Array(*v(2).downcast().unwrap()).into_value();
            ("{" (ws::ws_ml) object_inner (ws::ws_ml) "}") => |v, _| Json::Object(*v(2).downcast().unwrap()).into_value();
        }

        list_inner /* Vec<Json> */ {
            () => |_, _| Vec::<Json>::new().into_value();
            (list_items) => |v, _| v(0);
        }

        list_items /* Vec<Json> */ {
            (json) => |v, _| vec![*v(0).downcast::<Json>().unwrap()].into_value();
            (list_items (ws::ws_ml) "," (ws::ws_ml) json) => |v, _| {
                let mut list = v(0).downcast::<Vec<Json>>().unwrap();

                list.push(*v(4).downcast::<Json>().unwrap());

                list
            };
        }

        object_inner /* HashMap<String, Json> */ {
            () => |_, _| HashMap::<String, Json>::new().into_value();
            (key_value_pairs)
        }

        key_value_pairs /* HashMap<String, Json> */ {
            (key_value_pair) => |v, _| {
                let mut map = HashMap::new();
                let (key, value) = *v(0).downcast::<(String, Json)>().unwrap();
                map.insert(key, value);

                map.into_value()
            };

            (key_value_pairs (ws::ws_ml) "," (ws::ws_ml) key_value_pair) => |v, _| {
                let mut map = v(0).downcast::<HashMap<String, Json>>().unwrap();
                let (key, value) = *v(4).downcast::<(String, Json)>().unwrap();
                map.insert(key, value);

                map
            };
        }

        key_value_pair /* (String, Json) */ {
            ((string::string) (ws::ws_ml) ":" (ws::ws_ml) json) => |v, _| {
                (*v(0).downcast::<String>().unwrap(), *v(4).downcast::<Json>().unwrap()).into_value()
            };
        }
    }
}
