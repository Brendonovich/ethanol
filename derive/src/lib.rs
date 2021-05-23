use proc_macro::{TokenStream};
use syn::{parse_macro_input, Data, DeriveInput, DataStruct, Fields, FieldsNamed, Type, Ident};

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

struct FilterFieldMeta<'a> {
    t: &'a Type,
    is_relation: bool,
    input_type: Ident,
    model_getter: String,
    struct_name: String,
    ops: Vec<&'a str>
}

fn type_filters<'a>(type_string: &str) -> Result<Vec<&'a str>, ()> {
    match type_string {
        "String" => Ok(vec!["equals", "contains", "hasPrefix", "hasSuffix"]),
        "i32" => Ok(vec!["equals", "lte", "lt", "gt", "gte"]),
        "i64" => Ok(vec!["equals", "lte", "lt", "gt", "gte"]),
        "bool" => Ok(vec!["equals"]),
        _ => Err(())
    }
}

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
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
        let t = &f.ty;

        let t_str = &quote!{#t}.to_string();

        let (is_relation, filters) = match type_filters(t_str){
            Ok(filters) => (false, filters),
            Err(()) => (true, vec!["some", "every"])
        };
        FilterFieldMeta {
            t,
            input_type: format_ident!("{}", t_str),
            is_relation,
            ops: filters,
            model_getter: field_name.to_string(),
            struct_name: format!("{}{}Field", model_name.to_string(), title_case(&field_name.to_string()))
        }
    }).collect::<Vec<FilterFieldMeta>>();

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

        let field_ops = meta.ops.iter().map(|&op| {
            let op_fn_name = format_ident!("{}", op);
            let op_enum_case = format_ident!("{}_{}", &meta.model_getter, op);

            if meta.is_relation {
                let enum_type = format_ident!("{}Operation", meta.input_type);
                quote!{
                    pub fn #op_fn_name(&self, v: Vec<#enum_type>) -> #operation_enum_name {
                        #operation_enum_name::#op_enum_case(v)
                    }
                }
            } else {
                let input_type = &meta.input_type;
                quote! {
                    pub fn #op_fn_name(&self, v: #input_type) -> #operation_enum_name {
                        #operation_enum_name::#op_enum_case(v)
                    }
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
        let field_cases = meta.ops.iter().map(|&op| {
            let case_name = format_ident!("{}_{}", &meta.model_getter, op);

            if meta.is_relation {
                let t = &meta.input_type;

                let operation_name = format_ident!("{}Operation", t);

                quote! {
                    #case_name(Vec<#operation_name>)
                }
            }
            else {
                let type_name = meta.t;
                quote! {
                    #case_name(#type_name)
                }
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

        impl #model_name {
            #(#field_struct_getters)*
        }

        pub struct #queries_struct_name { }

        impl #queries_struct_name {
            fn find_one(&self, operations: Vec<#operation_enum_name>) -> Result<#model_name, ()> {
                // println!("{:?}", operations);
                Err(())
            }

            fn find_many(&self, operations: Vec<#operation_enum_name>) -> Result<Vec<#model_name>, ()> {
                // println!("{:?}", operations);
                Err(())
            }

            fn find_unique(&self, operations: Vec<#operation_enum_name>) -> Result<#model_name, ()> {
                // println!("{:?}", operations);
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
