extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemImpl, ImplItem, AttributeArgs, NestedMeta, Meta};

#[proc_macro_attribute]
pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}

#[proc_macro_attribute]
pub fn contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let struct_name = if let syn::Type::Path(type_path) = &*input.self_ty {
        &type_path.path.segments.first().unwrap().ident
    } else {
        panic!("Expected a struct.");
    };

    let mut public_methods = Vec::new();

    // Iterate over the items in the impl block to find pub methods
    for item in input.items.iter() {
        if let ImplItem::Method(method) = item {
            if method.vis == syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::token::Pub::default(),
            }) {
                public_methods.push(method.clone());
            }
        }
    }

    // Generate match arms for public methods
    let match_arms: Vec<_> = public_methods.iter().enumerate().map(|(index, method)| {
        let method_name = &method.sig.ident;
        //let method_selector = format!("{:08x}", index); // You might want to replace this with actual ABI selector calculation
        let method_selector = index as u8;
        let args: Vec<_> = method.sig.inputs.iter().skip(1).enumerate().map(|(i, _)| {
            let arg_name = format_ident!("arg{}", i);
            quote! { let #arg_name = *iter.next().unwrap() as u64; }
        }).collect();

        let arg_names: Vec<_> = (0..method.sig.inputs.len() - 1).map(|i| format_ident!("arg{}", i)).collect();

        quote! {
            #method_selector => {
                #( #args )*
                self.#method_name(#( #arg_names ),*);
            }
        }
    }).collect();

    // Generate the call method implementation
    let call_method = quote! {
        impl Contract for #struct_name {
            fn call(&self) {
                let address: usize = 0x1000; // Replace with actual address
                let length: usize = 32; // Replace with actual length
                let calldata = unsafe { slice_from_raw_parts(address, length) };

                let mut iter = calldata.iter();
                let selector = *iter.next().unwrap();
                match selector {
                    #( #match_arms )*
                    _ => revert(),
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn _start()
        {
            let contract = #struct_name::default();
            contract.call();
        }
    };

    let output = quote! {
        #input
        #call_method
    };

    TokenStream::from(output)
}
