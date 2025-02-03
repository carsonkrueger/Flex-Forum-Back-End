use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, Path, Token,
};

struct SchemaTableArgs {
    schema: Path,
    _comma: Token![,],
    table: Path,
}

impl Parse for SchemaTableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            schema: input.parse()?,
            _comma: input.parse()?,
            table: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn schema_table_def(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute arguments
    let args = parse_macro_input!(attr as SchemaTableArgs);
    let schema = args.schema;
    let table = args.table;

    // Parse the struct definition
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = input.ident.clone();

    let expanded = quote! {
        use sea_query::IntoIden;

        #input

        impl crate::model::schema::IntoSchemaTableRef for #struct_name {
            fn schema_table_ref() -> sea_query::TableRef {
                sea_query::TableRef::SchemaTable(#schema.into_iden(), #table.into_iden())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn iterator_iden_def(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the identifier (Iden struct) from the macro input
    let iden_struct = parse_macro_input!(attr as Path);

    // Convert item TokenStream to a syntax tree node
    let input: DeriveInput = parse_macro_input!(item);

    let struct_name = &input.ident;

    // Extract struct fields
    let fields = match input.data {
        syn::Data::Struct(ref data_struct) => match &data_struct.fields {
            syn::Fields::Named(ref named_fields) => named_fields,
            _ => {
                return TokenStream::from(quote! {
                    compile_error!("Expected named fields in the struct");
                })
                .into();
            }
        },
        _ => {
            return TokenStream::from(quote! {
                compile_error!("Expected a struct with named fields");
            })
            .into();
        }
    };

    // Generate identifier references for each field
    let column_idens: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap().to_string();
            let pascal_case_name = to_pascal_case(&field_name);
            format_ident!("{}", pascal_case_name)
        })
        .collect();

    let expanded = quote! {
        #input

        impl crate::model::schema::IntoIteratorIden for #struct_name
        {
            type C = #iden_struct;
            type IC = std::vec::IntoIter<Self::C>;
            fn into_iterator_iden() -> Self::IC {
                vec![
                    #(
                        #iden_struct::#column_idens,
                    )*
                ].into_iter()
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn iterator_def(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Convert item TokenStream to a syntax tree node
    let input: DeriveInput = parse_macro_input!(item);

    let struct_name = &input.ident;

    // Extract struct fields
    let fields = match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(named_fields) => &named_fields.named,
            _ => {
                return quote! {
                    compile_error!("Expected a struct with named fields");
                }
                .into();
            }
        },
        _ => {
            return quote! {
                compile_error!("Expected a struct");
            }
            .into();
        }
    };

    // Generate field access for struct values and corresponding identifiers
    let column_exprs = fields.iter().map(|f| f.ident.as_ref().unwrap());

    let expanded = quote! {
        #input

        impl crate::model::schema::IntoIteratorExprVal for #struct_name {
            type IntoIter = std::vec::IntoIter<sea_query::SimpleExpr>;

            fn into_iterator_val(&self) -> Self::IntoIter {
                vec![
                    #(
                        self.#column_exprs.clone().into(),
                    )*
                ]
                .into_iter()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
