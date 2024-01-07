use super::*;

declare_rules! {
    pub Boolean {
        boolean {
            ("true") => |_| true.into_value();
            ("false") => |_| false.into_value();
        }
    }
}
