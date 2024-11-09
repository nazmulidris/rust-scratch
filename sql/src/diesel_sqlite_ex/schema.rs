// @generated automatically by Diesel CLI.

diesel::table! {
    data_table (id) {
        id -> Text,
        name -> Text,
        data -> Text,
    }
}

diesel::table! {
    file_table (id) {
        id -> Text,
        name -> Text,
        data -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    data_table,
    file_table,
);
