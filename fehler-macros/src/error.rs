use quote::quote;

pub fn entry(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let source_body = s.each_variant(|v| {
        let mut sources = v.bindings().iter().filter(is_source);
        match (sources.next(), sources.next()) {
            (Some(source), None)    => quote!({return Some(fehler::AsError::as_error(#source))}),
            (None, None)            => quote!({return None}),
            (_, Some(_))            => panic!("cannot have multiple source attributes"),
        }
    });

    // This is done to avoid always requiring users turn on the backtrace feature
    let mut used_backtrace = false;

    let backtrace_body = s.each_variant(|v| {
        let mut backtraces = v.bindings().iter().filter(is_backtrace);
        match (backtraces.next(), backtraces.next()) {
            (Some(backtrace), None) => {
                used_backtrace = true;
                quote!({return Some(#backtrace)})
            }
            (None, None)            => quote!({return None}),
            (_, Some(_))            => panic!("cannot have multiple backtraces"),
        }
    });

    let backtrace_fn = match used_backtrace {
        true    => quote! {
            fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
                match *self { #backtrace_body }
            }
        },
        false   => quote! { }
    };

    s.unbound_impl(quote!(std::error::Error), quote! {
        #backtrace_fn

        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match *self { #source_body }
        }

        fn cause(&self) -> Option<&dyn std::error::Error> {
            match *self { #source_body }
        }
    })
}

fn is_source(b: &&synstructure::BindingInfo) -> bool {
    let mut source_attrs = 0;
    for attr in &b.ast().attrs {
        if let Ok(meta) = attr.parse_meta() {
            if meta.path().is_ident("error") {
                if let syn::Meta::List(list) = &meta {
                    for nested in &list.nested {
                        if let syn::NestedMeta::Meta(meta) = nested {
                            if meta.path().is_ident("cause") || meta.path().is_ident("source") {
                                source_attrs += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    match source_attrs {
        0   => false,
        1   => true,
        _   => panic!("cannot have multiple source attributes")
    }
}

fn is_backtrace(b: &&synstructure::BindingInfo) -> bool {
    if let syn::Type::Path(syn::TypePath { path, .. }) = &b.ast().ty {
        if let Some(segment) = path.segments.last() {
            segment.ident == "Backtrace"
        } else { false }
    } else { false }
}
