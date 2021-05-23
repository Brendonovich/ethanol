use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, DataStruct, Fields, FieldsNamed,  Type};

#[macro_use]
extern crate quote;
extern crate proc_macro;

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

struct FieldMeta {
    t: Type,
    model_getter: String,
    struct_name: String,
}

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let ops = vec!["equals", "contains"];
    let input = parse_macro_input!(input as DeriveInput);
    let model_name = input.ident;

    let queries_struct_name = format_ident!("{}Queries", model_name);
    let operation_enum_name = format_ident!("{}Operation", model_name);
    let client_fn_name = format_ident!("{}", model_name.to_string().to_ascii_lowercase());
    let client_trait_name = format_ident!("{}Client", model_name);

    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed {
            ref named,
            ..
        }),
        ..
    }) = input.data {
        named
    } else {
        unimplemented!()
    };

    let field_metas = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let t = f.ty.clone();

        FieldMeta {
            t,
            model_getter: field_name.to_string(),
            struct_name: format!("{}{}Field", model_name.to_string(), title_case(&field_name.to_string()))
        }
    }).collect::<Vec<FieldMeta>>();

    let field_struct_declarations = field_metas.iter().map(|meta| {
        let field_struct_name = format_ident!("{}", meta.struct_name);
        quote! {
            pub struct #field_struct_name {}
        }
    });

    let field_struct_getters = field_metas.iter().map(|meta| {
        let model_getter = format_ident!("{}", meta.model_getter);
        let struct_name = format_ident!("{}", meta.struct_name);

        quote! {
            pub fn #model_getter() -> #struct_name {
                #struct_name { }
            }
        }
    });

    let field_struct_impls = field_metas.iter().map(|meta| {
        let field_struct_name = format_ident!("{}", meta.struct_name);

        let field_ops = ops.iter().map(|&op| {
            let op_fn_name = format_ident!("{}", op);
            let op_enum_case = format_ident!("{}{}", title_case(&meta.model_getter), title_case(op));
            let field_type = &meta.t;

            quote! {
                pub fn #op_fn_name(&self, v: #field_type) -> #operation_enum_name {
                    #operation_enum_name::#op_enum_case(v)
                }
            }
        });

        quote! {
            impl #field_struct_name {
                #(#field_ops)*
            }
        }
    });

    let operation_enum_cases = field_metas.iter().map(|meta| {
        let t = &meta.t;
        let field_name = &title_case(&meta.model_getter);

        let field_cases = ops.iter().map(|&op| {
            let case_name = format_ident!("{}{}", field_name, title_case(op));

            quote! {
                #case_name(#t)
            }
        });

        quote! {
            #(#field_cases),*
        }
    });

    let m = quote! {
        #[derive(Debug)]
        pub enum #operation_enum_name {
            #(#operation_enum_cases),*
        }

        #(#field_struct_declarations)*

        #(#field_struct_impls)*

        impl Account {
            #(#field_struct_getters)*
        }

        pub struct #queries_struct_name { }

        impl #queries_struct_name {
            fn find_one(&self, operations: Vec<#operation_enum_name>) -> Result<#model_name, ()> {
                Err(())
            }
        }

        trait #client_trait_name {
            fn #client_fn_name(&self) -> #queries_struct_name;
        }

        impl #client_trait_name for Client {
            fn #client_fn_name(&self) -> #queries_struct_name {
                #queries_struct_name { }
            }
        }
    };

    TokenStream::from(m)
}
