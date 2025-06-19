pub mod definitions_read_model;
pub mod schema_projection;

pub use schema_projection::{
    flatten_json_schema, generate_create_table_statement, generate_index_statements,
    FlattenedAttribute,
};
