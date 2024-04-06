use super::*;

declare_rules! {
    pub Boolean {
        boolean {
            ("true") => |_, _| true.into_value();
            ("false") => |_, _| false.into_value();
        }
    }
}
