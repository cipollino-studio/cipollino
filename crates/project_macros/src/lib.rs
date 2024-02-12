
use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

fn field_has_attr(field: &Field, attr: &str) -> bool {
    let attr = format!("#[{}]", attr);
    for other_attr in &field.attrs {
        if other_attr.to_token_stream().to_string() == attr {
            return true;
        } 
    }
    return false;
}

#[proc_macro_derive(Object, attributes(field))]
pub fn object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let fields = if let Data::Struct(data) = ast.data {
        data.fields
    } else {
        panic!("object must be a struct!");
    };
    let name = ast.ident;
    let list_name = Ident::new((name.to_string().to_ascii_lowercase() + "s").as_str(), name.span()); 

    let mut field_setters = quote!{};
    for field in fields {
        let field_name = field.ident.clone();
        let ty = field.ty.to_token_stream();

        if field_has_attr(&field, "field") {
            let setter_name = format_ident!("set_{}", field_name.clone().to_token_stream().to_string());
            field_setters.append_all(quote! {
                pub fn #setter_name(project: &mut Project, ptr: ObjPtr<Self>, #field_name: #ty) -> Option<ObjAction> {
                    project.#list_name.get_then_mut(ptr, |obj| {
                        let init_val = obj.#field_name.clone();
                        obj.#field_name = #field_name.clone();
                        ObjAction::new(move |proj| {
                            #name::#setter_name(proj, ptr, #field_name.clone());
                        }, move |proj| {
                            #name::#setter_name(proj, ptr, init_val.clone());
                        })
                    })
                } 
            });
        }
    }

    quote! {
        impl Obj for #name {

            fn get_list(project: &Project) -> &ObjList<Self> {
                &project.#list_name
            }

            fn get_list_mut(project: &mut Project) -> &mut ObjList<Self> {
                &mut project.#list_name
            }

        } 

        impl #name {

            #field_setters 
            
        }

    }.into()
}

#[proc_macro_derive(ObjClone, attributes(parent))]
pub fn obj_clone(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let fields = if let Data::Struct(data) = ast.data {
        data.fields
    } else {
        panic!("object must be a struct!");
    };
    let name = ast.ident;

    let mut obj_clone_impl = quote!{};
    for field in fields {
        let field_name = field.ident.clone();
        obj_clone_impl.append_all(quote! {
            #field_name: self.#field_name.obj_clone(project),
        });

        
    }

    quote! {

        impl ObjClone for #name {

            fn obj_clone(&self, project: &mut Project) -> Self {
                Self {
                    #obj_clone_impl
                }
            }

        }

    }.into()
}

#[proc_macro_derive(ObjSerialize, attributes(parent))]
pub fn obj_serialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let fields = if let Data::Struct(data) = ast.data {
        data.fields
    } else {
        panic!("object must be a struct!");
    };
    let name = ast.ident;

    let mut serialize_impl = quote!{};
    let mut deserialize_impl = quote!{};
    for field in fields {
        let field_name = field.ident.clone();
        if !field_has_attr(&field, "parent") {
            let field_name_str = field_name.to_token_stream().to_string();
            serialize_impl.append_all(quote! {
                #field_name_str: self.#field_name.obj_serialize(project),
            });

            let ty = field.ty.to_token_stream();
            deserialize_impl.append_all(quote! {
                if let Some(field) = data.get(#field_name_str) {
                    if let Some(val) = <#ty>::obj_deserialize(project, field, parent) {
                        res.#field_name = val;
                    }
                }
            });
        } else {
            deserialize_impl.append_all(quote! {
                res.#field_name = parent.into();
            });
        }
    }

    quote! {
        
        impl ObjSerialize for #name {

            fn obj_serialize(&self, project: &Project) -> serde_json::Value {
                serde_json::json! {{
                    #serialize_impl
                }}
            }

            fn obj_deserialize(project: &mut Project, data: &serde_json::Value, parent: ObjPtrAny) -> Option<Self> {
                let mut res = Self::default();
                #deserialize_impl 
                Some(res)
            }

        }
    }.into()

}