use super::*;

declare_rules! {
    pub Whitespace {
        // single line whitespace
        // matches nothing or any number of spaces or tabs
        ws {
            ()
            (ws " ")
            (ws "\t")
        }

        // multiline whitespace
        // matches nothing or any number of spaces, tabs, or newlines
        ws_ml {
            ()
            (ws_ml " ")
            (ws_ml "\t")
            (ws_ml "\r")
            (ws_ml "\n")
        }
    }
}
