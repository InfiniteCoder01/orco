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
    let ast: syn::ItemImpl = parse_macro_input!(item);

    let generics = &ast.generics;
    let r#type = &ast.self_ty;

    quote! {
        #ast

        impl #generics std::fmt::Debug for #r#type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <Self as std::fmt::Display>::fmt(self, f)
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn make_mut(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = parse_macro_input!(item);
    let mut ast2 = ast.clone();

    ast2.sig.ident = syn::Ident::new(&format!("{}_mut", ast.sig.ident), ast.sig.ident.span());
    let syn::FnArg::Receiver(arg) = ast2.sig.inputs.first_mut().unwrap() else {
        panic!();
    };
    *arg = syn::parse2(quote![&mut self]).unwrap();

    let syn::ReturnType::Type(_, rt) = &mut ast2.sig.output else {
        panic!();
    };

    let syn::Type::Path(path) = &mut **rt else {
        panic!();
    };

    let seg = path.path.segments.last_mut().unwrap();
    seg.arguments =
        syn::PathArguments::AngleBracketed(syn::parse2(quote! { <orco::Mut> }).unwrap());

    quote! {
        #ast
        #ast2
    }
    .into()
}

#[proc_macro_derive(MutrefCloneCopy)]
pub fn mutref_clonecopy(item: TokenStream) -> TokenStream {
    use syn::punctuated::Punctuated;

    let ast: DeriveInput = parse_macro_input!(item);
    let syn::Data::Enum(data) = ast.data else {
        panic!()
    };

    let name = ast.ident;
    let mut arms = Punctuated::new();

    for variant in data.variants {
        let name = variant.ident;
        let mut fields_pat = Punctuated::new();
        let mut fields_cloned = Punctuated::new();
        for i in 0..variant.fields.len() {
            let field_name = syn::Ident::new(&format!("item{}", i), proc_macro2::Span::call_site());
            fields_pat.push_value(quote![#field_name]);
            fields_pat.push_punct(quote![,]);
            fields_cloned.push_value(quote![#field_name.clone()]);
            fields_cloned.push_punct(quote![,]);
        }
        arms.push_value(quote! {
            Self::#name(#fields_pat) => Self::#name(#fields_cloned)
        });
        arms.push_punct(quote![,]);
    }

    quote! {
        impl Clone for #name<'_, Imm> {
            fn clone(&self) -> Self {
                match self {
                    #arms
                }
            }
        }

        impl Copy for Expression<'_, Imm> {}
    }
    .into()
}
