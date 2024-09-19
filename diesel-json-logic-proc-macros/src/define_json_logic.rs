use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Path, Token, Type,
};

struct JsonLogicInput {
    ident: Ident,
    columns: Vec<QueryColumn>,
}

struct QueryColumn {
    query_column_name: QueryColumnName,
    diesel_column_name: Path,
    ty: Type,
}

struct QueryColumnName(Ident);

impl QueryColumnName {
    fn capitalize_word(word: &str) -> String {
        let mut c = word.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    fn as_camel_case(&self) -> Ident {
        Ident::new(
            &self
                .0
                .to_string()
                .replace(" ", "")
                .split("_")
                .map(Self::capitalize_word)
                .collect::<String>(),
            self.0.span(),
        )
    }

    fn as_snake_case(&self) -> Ident {
        self.0.clone()
    }

    fn as_struct_field_name(&self) -> Ident {
        self.as_snake_case()
    }

    fn as_column_struct_name(&self) -> Ident {
        self.as_camel_case()
    }

    fn as_var_struct_name(&self) -> Ident {
        format_ident!("{}Var", self.as_camel_case())
    }
}

impl Parse for QueryColumn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![#]>()?;
        let content;
        bracketed!(content in input);
        let diesel_column_name: Ident = content.parse()?;
        if !diesel_column_name.eq("diesel_column_name") {
            return Err(syn::Error::new(
                diesel_column_name.span(),
                "Expected attribute 'diesel_column_name'",
            ));
        }
        content.parse::<Token![=]>()?;
        let diesel_column_name: Path = content.parse()?;

        let query_column_name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let ty: Type = input.parse()?;
        Ok(QueryColumn {
            query_column_name: QueryColumnName(query_column_name),
            diesel_column_name,
            ty,
        })
    }
}

impl Parse for JsonLogicInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        bracketed!(content in input);
        let columns = content.parse_terminated(QueryColumn::parse, Token![,])?;

        Ok(JsonLogicInput {
            ident,
            columns: columns.into_iter().collect(),
        })
    }
}

fn generate_column_struct(query_column_name: &QueryColumnName) -> TokenStream {
    let struct_name = query_column_name.as_column_struct_name();
    let quoted_struct_name = struct_name.to_string();
    let column_path_string = query_column_name.as_struct_field_name().to_string();
    quote! {
        #[allow(dead_code)]
        #[derive(Debug, ::serde::Deserialize, PartialEq)]
        #[serde(remote = #quoted_struct_name)]
        pub struct #struct_name(String);

        impl<'de> ::serde::Deserialize<'de> for #struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let contents = String::deserialize(deserializer)?;
                if contents == #column_path_string {
                    Ok(Self(contents))
                } else {
                    Err(::serde::de::Error::custom(format!(
                        "{} can only receive {}, but instead received {contents}",
                        #quoted_struct_name,
                        #column_path_string
                    )))
                }

            }
        }
    }
}

fn generate_var_struct(
    query_column_name: &QueryColumnName,
    diesel_column_name: &Path,
) -> TokenStream {
    let struct_name = query_column_name.as_var_struct_name();
    let column_struct_name = query_column_name.as_column_struct_name();
    quote! {
        #[derive(Debug, ::serde::Deserialize, PartialEq)]
        pub struct #struct_name {
            pub var: #column_struct_name,
        }

        impl #struct_name {
            pub fn to_diesel_column(&self) -> #diesel_column_name {
                #diesel_column_name
            }
        }

    }
}

fn generate_query_struct(struct_name: &Ident, column_names: &[QueryColumn]) -> TokenStream {
    let fields: Vec<TokenStream> = column_names
        .iter()
        .map(
            |QueryColumn {
                 query_column_name,
                 ty,
                 ..
             }| {
                let var_struct_name = query_column_name.as_var_struct_name();
                let field_name = query_column_name.as_struct_field_name();
                quote! {
                    #field_name: Option<JsonLogicExpr<#var_struct_name, #ty>>
                }
            },
        )
        .collect();
    quote! {
        #[derive(Debug, ::serde::Deserialize, PartialEq)]
        pub struct #struct_name{
            #(pub #fields),*
        }
    }
}

fn camel_to_snake(s: String) -> String {
    let mut snake_case = String::with_capacity(s.len());
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                snake_case.push('_');
            }
            snake_case.push(ch.to_lowercase().next().unwrap());
        } else {
            snake_case.push(ch);
        }
    }
    snake_case
}

fn generate_query_unpacking_macro(ident: Ident, columns: Vec<QueryColumn>) -> TokenStream {
    let field_names: Vec<Ident> = columns
        .iter()
        .map(
            |QueryColumn {
                 query_column_name, ..
             }| query_column_name.as_struct_field_name(),
        )
        .collect();
    let macro_name = Ident::new(
        &format!("impl_{}", camel_to_snake(ident.to_string())),
        Span::call_site(),
    );
    quote! {macro_rules! #macro_name {
            ($query:ident, $from_stmt:ident) => {
                diesel_json_logic_macro_rules::unpack_json_logic_query!([#(#field_names),*], $query, $from_stmt)
            };
        }
    }
}

pub fn define_json_logic(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let JsonLogicInput { ident, columns } = parse_macro_input!(tokens as JsonLogicInput);
    let structs: Vec<TokenStream> = columns
        .iter()
        .map(
            |QueryColumn {
                 query_column_name,
                 diesel_column_name,
                 ..
             }| {
                let column = generate_column_struct(query_column_name);
                let var = generate_var_struct(query_column_name, diesel_column_name);
                quote! {
                    #column
                    #var
                }
            },
        )
        .collect();

    let query_struct = generate_query_struct(&ident, &columns);
    let query_unpacking_macro = generate_query_unpacking_macro(ident, columns);

    quote! {
        #(#structs)*

        #query_struct

        #query_unpacking_macro
    }
    .into()
}
