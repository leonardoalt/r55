extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ImplItem, ItemImpl};

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
            if method.vis
                == syn::Visibility::Public(syn::VisPublic {
                    pub_token: syn::token::Pub::default(),
                })
            {
                public_methods.push(method.clone());
            }
        }
    }

    // Generate match arms for public methods
    let match_arms: Vec<_> = public_methods.iter().enumerate().map(|(index, method)| {
        let method_name = &method.sig.ident;
        let method_selector = index as u32;
        let arg_types: Vec<_> = method.sig.inputs.iter().skip(1).map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                let ty = &*pat_type.ty;
                quote! { #ty }
            } else {
                panic!("Expected typed arguments");
            }
        }).collect();

        let arg_names: Vec<_> = (0..method.sig.inputs.len() - 1).map(|i| format_ident!("arg{}", i)).collect();

        quote! {
            #method_selector => {
                let (#( #arg_names ),*) = <(#( #arg_types ),*)>::abi_decode(calldata, true).unwrap();
                self.#method_name(#( #arg_names ),*);
            }
        }
    }).collect();

    // Generate the call method implementation
    let call_method = quote! {
        use alloy_sol_types::SolValue;
        use eth_riscv_runtime::{revert, return_riscv, slice_from_raw_parts, Contract};

        impl Contract for #struct_name {
            fn call(&self) {
                let address: usize = 0x8000_0000;
                let length = unsafe { slice_from_raw_parts(address, 4) };
                let length = u32::from_le_bytes([length[0], length[1], length[2], length[3]]) as usize;
                let calldata = unsafe { slice_from_raw_parts(address + 4, length) };
                self.call_with_data(calldata);
            }

            fn call_with_data(&self, calldata: &[u8]) {
                let selector = u32::from_le_bytes([calldata[0], calldata[1], calldata[2], calldata[3]]);
                let calldata = &calldata[4..];

                match selector {
                    #( #match_arms )*
                    _ => revert(),
                }

                return_riscv(0, 0);
            }
        }

        #[eth_riscv_runtime::entry]
        fn main() -> !
        {
            let contract = #struct_name::default();
            contract.call();
            eth_riscv_runtime::return_riscv(0, 0)
        }
    };

    let output = quote! {
        #input
        #call_method
    };

    TokenStream::from(output)
}
