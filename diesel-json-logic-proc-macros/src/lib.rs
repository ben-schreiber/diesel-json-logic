mod define_json_logic;

use proc_macro::TokenStream;

/// Use this macro to generate the necessary code to convert
/// a JSON Logic query to a diesel filter statement.
///
/// The macro expects two arguments:
///   1. The name of the struct to house the JSON Logic queries.
///   2. A bracketed list of columns and their types which can be used in the filters.
///
/// For example:
/// ```ignore
/// define_json_logic!(
///     MyTableQuery,
///     [
///        #[diesel_column_name = my_tbl::best_column]
///        best_column => i32,
///        #[diesel_column_name = my_tbl::second_best_column]
///        other_column => String  
///     ]
/// )
/// ```
///
/// The above macro call generates the following objects:
///   1. `MyTableQuery` - the main struct, deserialized from the API request, which contains
///         the JSON Logic queries - if any.
///   2. `BestColumnVar` (& `OtherColumnVar`) - struct with a single field named `var` which points
///         to another struct named `BestColumn` (`OtherColumn`). This struct is a container for the
///         `my_tbl::best_column` (`my_tbl::second_best_column`) column.
///   3. `impl_my_table_query` - A macro which translates the JSON Logic queries in `MyTableQuery` to
///         `diesel` SQL query filters.
///
/// See here for the JSON Logic spec -> https://jsonlogic.com/
#[proc_macro]
pub fn define_json_logic(tokens: TokenStream) -> TokenStream {
    self::define_json_logic::define_json_logic(tokens)
}
