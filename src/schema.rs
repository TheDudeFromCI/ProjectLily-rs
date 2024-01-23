// @generated automatically by Diesel CLI.

diesel::table! {
    chat_log (id) {
        id -> Integer,
        role -> Text,
        user -> Nullable<Text>,
        content -> Text,
        action -> Nullable<Text>,
        severity -> Nullable<Text>,
        time -> Timestamp,
        token_count -> SmallInt,
    }
}
