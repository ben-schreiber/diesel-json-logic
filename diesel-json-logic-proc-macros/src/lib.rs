mod define_json_logic;

use proc_macro::TokenStream;

#[proc_macro]
pub fn define_json_logic(tokens: TokenStream) -> TokenStream {
    self::define_json_logic::define_json_logic(tokens)
}
