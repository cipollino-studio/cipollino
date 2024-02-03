
use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, Ident};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

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

    let mut obj_clone_impl = quote!{};
    let mut field_setters = quote!{};
    for field in fields {
        let field_name = field.ident;
        let ty = field.ty.to_token_stream();

        obj_clone_impl.append_all(quote! {
            #field_name: self.#field_name.obj_clone(project),
        });

        for attr in field.attrs {
            // Hack
            if attr.to_token_stream().to_string() == "#[field]" {
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

        impl ObjClone for #name {
            
            fn obj_clone(&self, project: &mut Project) -> Self {
                Self {
                    #obj_clone_impl
                }                
            }

        }

        impl #name {

            #field_setters 
            
        }

    }.into()
}
