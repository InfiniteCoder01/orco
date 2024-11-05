use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Unit, attributes(display))]
pub fn derive_unit(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(item);
    let display = display(&ast, quote! {orco::Unit});
    let name = ast.ident;

    quote! {
        impl orco::Unit for #name {
            fn symbols<'a>(&'a self) -> orco::DynIter<'a, &'a orco::Symbol> {
                Box::new(self.symbols.iter())
            }

            fn symbols_mut<'a>(&'a mut self) -> orco::DynIter<'a, &'a mut orco::Symbol> {
                Box::new(self.symbols.iter_mut())
            }
        }

        #display
    }
    .into()
}

#[proc_macro_attribute]
pub fn helper(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast: syn::ItemImpl = parse_macro_input!(item);
    assert_eq!(
        ast.trait_
            .as_ref()
            .unwrap()
            .1
            .segments
            .first()
            .unwrap()
            .ident,
        "orco"
    );
    let r#struct = &ast.self_ty;
    let r#trait = &ast.trait_.as_ref().unwrap().1;

    let mut display_impl = quote! {};
    ast.attrs.retain(|attr| {
        let getters = [("symbols", (true, "orco::Symbol"))]
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();
        if attr.path().is_ident("display") {
            display_impl = display(r#struct, r#trait);
            return false;
        } else if let (Some(name), Ok(field)) =
            (attr.path().get_ident(), attr.parse_args::<syn::Ident>())
        {
            if let Some((vec, r#type)) = getters.get(&name.to_string().as_str()) {
                let name_mut = syn::parse_str::<syn::Ident>(&format!("{}_mut", name)).unwrap();
                let r#type = syn::parse_str::<syn::Type>(r#type).unwrap();
                let funcs = if *vec {
                    [
                        quote! {
                            fn #name<'a>(&'a self) -> orco::DynIter<'a, &'a #r#type> {
                                Box::new(self.#field.iter())
                            }
                        },
                        quote! {
                            fn #name_mut<'a>(&'a mut self) -> orco::DynIter<'a, &'a mut #r#type> {
                                Box::new(self.#field.iter_mut())
                            }
                        },
                    ]
                } else {
                    [
                        quote! {
                            fn #name(&self) -> &#r#type {
                                &self.#field
                            }
                        },
                        quote! {
                            fn #name_mut(&mut self) -> &mut #r#type {
                                &mut self.#field
                            }
                        },
                    ]
                };
                ast.items
                    .extend(funcs.into_iter().map(|func| syn::parse2(func).unwrap()));
                return false;
            }
        }

        true
    });

    quote! {
        #ast
        #display_impl
    }
    .into()
}

fn display(
    name: impl quote::ToTokens,
    trait_name: impl quote::ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                (self as &dyn #trait_name).fmt(f)
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                (self as &dyn #trait_name).fmt(f)
            }
        }
    }
}

#[proc_macro_attribute]
pub fn debug_display(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::Item = parse_macro_input!(item);
    let name = match &ast {
        syn::Item::Const(item) => &item.ident,
        syn::Item::Enum(item) => &item.ident,
        syn::Item::ExternCrate(item) => &item.ident,
        syn::Item::Mod(item) => &item.ident,
        syn::Item::Static(item) => &item.ident,
        syn::Item::Struct(item) => &item.ident,
        syn::Item::Trait(item) => &item.ident,
        syn::Item::TraitAlias(item) => &item.ident,
        syn::Item::Type(item) => &item.ident,
        syn::Item::Union(item) => &item.ident,
        _ => panic!("Can't put Debug on that!"),
    };

    let d = if matches!(ast, syn::Item::Trait(_)) {
        quote! { dyn }
    } else {
        quote! {}
    };
    quote! {
        #ast

        impl std::fmt::Debug for #d #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <Self as std::fmt::Display>::fmt(self, f)
            }
        }
    }
    .into()
}
