
use convert_case::Casing;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Field, Ident};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

fn attr_list_contains(attrs: &Vec<Attribute>, attr: &str) -> bool {
    let attr = format!("#[{}]", attr);
    for other_attr in attrs {
        if other_attr.to_token_stream().to_string() == attr {
            return true;
        } 
    }
    false
}

fn field_has_attr(field: &Field, attr: &str) -> bool {
    attr_list_contains(&field.attrs, attr) 
}

#[proc_macro_derive(Object, attributes(field, asset))]
pub fn object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let fields = if let Data::Struct(data) = ast.data {
        data.fields
    } else {
        panic!("object must be a struct!");
    };
    let name = ast.ident;
    let list_name = Ident::new((name.to_string() + "s").to_case(convert_case::Case::Snake).as_str(), name.span()); 
    let type_name = name.to_string().to_case(convert_case::Case::Lower);

    let is_asset = attr_list_contains(&ast.attrs, "asset");

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

    let list_type = if is_asset {
        quote! { crate::project::obj::asset_list::AssetList<Self> }
    } else {
        quote! { crate::project::obj::obj_list::ObjList<Self> }
    };

    quote! {
        impl Obj for #name {

            type ListType = #list_type;

            fn get_list(project: &Project) -> &Self::ListType {
                &project.#list_name
            }

            fn get_list_mut(project: &mut Project) -> &mut Self::ListType {
                &mut project.#list_name
            }

            fn type_name() -> &'static str {
                #type_name 
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

    let raw_data_name = format_ident!("{}RawData", name);

    let mut serialize_impl = quote!{};
    let mut serialize_full_impl = quote!{};
    let mut deserialize_impl = quote!{};
    let mut raw_data_struct_fields = quote!{};
    let mut to_raw_data_impl = quote!{};
    let mut from_raw_data_impl = quote!{};
    for field in fields {
        let field_name = field.ident.clone();
        let ty = field.ty.to_token_stream();

        if !field_has_attr(&field, "parent") {
            let field_name_str = field_name.to_token_stream().to_string();
            serialize_impl.append_all(quote! {
                #field_name_str: self.#field_name.obj_serialize(project, asset_file),
            });

            serialize_full_impl.append_all(quote! {
                #field_name_str: self.#field_name.obj_serialize_full(project, asset_file),
            });

            deserialize_impl.append_all(quote! {
                if let Some(field) = crate::util::bson::bson_get(data, #field_name_str) {
                    if let Some(val) = <#ty>::obj_deserialize(project, field, parent, asset_file, metadata) {
                        res.#field_name = val;
                    }
                }
            });
        } 

        raw_data_struct_fields.append_all(quote! {
            #field_name: <#ty as crate::project::obj::ToRawData>::RawData,
        });

        to_raw_data_impl.append_all(quote! {
            #field_name: self.#field_name.to_raw_data(project),
        });

        from_raw_data_impl.append_all(quote! {
            #field_name: <#ty as crate::project::obj::ToRawData>::from_raw_data(project, &data.#field_name),
        });
    }

    quote! {

        
        impl ObjSerialize for #name {

            fn obj_serialize(&self, project: &Project, asset_file: &mut crate::project::saveload::asset_file::AssetFile) -> bson::Bson {
                bson::bson! {{
                    #serialize_impl
                }}
            }

            fn obj_serialize_full(&self, project: &Project, asset_file: &mut crate::project::saveload::asset_file::AssetFile) -> bson::Bson {
                bson::bson! {{
                    #serialize_full_impl
                }}
            }

            fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: crate::project::obj::DynObjPtr, asset_file: &mut crate::project::saveload::asset_file::AssetFile, metadata: &mut crate::project::saveload::load::LoadingMetadata) -> Option<Self> {
                let mut res = Self::default();
                #deserialize_impl 
                Some(res)
            }

        }

        pub struct #raw_data_name {
            #raw_data_struct_fields
        }

        impl crate::project::obj::ToRawData for #name {

            type RawData = #raw_data_name;

            fn to_raw_data(&self, project: &Project) -> Self::RawData {
                Self::RawData {
                    #to_raw_data_impl
                }
            }

            fn from_raw_data(project: &mut Project, data: &Self::RawData) -> Self {
                Self {
                    #from_raw_data_impl
                }
            }

        }
    }.into()

}
