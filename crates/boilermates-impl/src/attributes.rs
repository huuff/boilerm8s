use quote::quote;
use syn::{parse_quote, Attribute, Meta, MetaList};

nestify::nest! {
  pub enum BoilermatesStructAttribute {
    AttrFor(pub struct BoilermatesAttrFor {
      pub target_struct: String,
      pub attribute: Attribute,
    })
  }
}

impl BoilermatesStructAttribute {
    pub fn extract(attributes: &mut Vec<Attribute>) -> Result<Vec<Self>, anyhow::Error> {
        use itertools::Itertools as _;

        let (boilermates_attrs, non_boilermates_attrs) = std::mem::take(attributes)
            .into_iter()
            .partition(is_boilermates);

        *attributes = non_boilermates_attrs;

        boilermates_attrs
            .into_iter()
            .map(|attr| match &attr.meta {
                Meta::List(list) => {
                    let meta: Meta = list.parse_args()?;
                    match meta {
                        Meta::List(attr) if attr.path.is_ident("attr_for") => {
                            Ok(BoilermatesStructAttribute::AttrFor(
                                syn::parse2(attr.tokens)?,
                            ))
                        }
                        _ => Err(anyhow::anyhow!(
                            "unknown boilermates attribute: `{}`",
                            quote::quote!(#meta)
                        )),
                    }
                }
                _ => Err(anyhow::anyhow!(
                    "unknown boilermates attribute: `{}`",
                    quote::quote!(#attr)
                )),
            })
            .try_collect()
    }
}

fn is_boilermates(attr: &Attribute) -> bool {
    attr.path().is_ident("boilermates")
}

impl syn::parse::Parse for BoilermatesAttrFor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target_struct: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let attrs: proc_macro2::TokenStream = input.parse()?;

        Ok(Self {
            target_struct: target_struct.to_string(),
            attribute: {
                // TODO: there must be a better way
                let q = quote! { #attrs };
                parse_quote! { # q }
            },
        })

    }
}

nestify::nest! {
  pub enum BoilermatesFieldAttribute {
    OnlyIn(pub struct BoilermatesOnlyIn(pub Vec<String>)),
    NotIn(pub struct BoilermatesNotIn(pub Vec<String>)),
    Default,
    OnlyInSelf,
  }
}

impl BoilermatesFieldAttribute {
    pub fn extract(
        attributes: &mut Vec<Attribute>,
    ) -> Result<Vec<BoilermatesFieldAttribute>, anyhow::Error> {
        use itertools::Itertools as _;

        let (boilermates_attrs, non_boilermates_attrs) = std::mem::take(attributes)
            .into_iter()
            .partition(is_boilermates);

        *attributes = non_boilermates_attrs;

        boilermates_attrs
            .into_iter()
            .map(|attr| match &attr.meta {
                Meta::List(list) => {
                    let meta: Meta = list.parse_args()?;
                    match meta {
                        Meta::List(list) if list.path.is_ident("only_in") => {
                            Ok(BoilermatesFieldAttribute::OnlyIn(BoilermatesOnlyIn(
                                extract_nested_list(&list)?,
                            )))
                        }
                        Meta::List(list) if list.path.is_ident("not_in") => {
                            Ok(BoilermatesFieldAttribute::NotIn(BoilermatesNotIn(
                                extract_nested_list(&list)?,
                            )))
                        }
                        Meta::Path(path) if path.is_ident("default") => {
                            Ok(BoilermatesFieldAttribute::Default)
                        }
                        Meta::Path(path) if path.is_ident("only_in_self") => {
                            Ok(BoilermatesFieldAttribute::OnlyInSelf)
                        }
                        _ => anyhow::bail!(
                            "unknown boilermates attribute: `{}`",
                            quote::quote!(#meta)
                        ),
                    }
                }
                _ => Err(anyhow::anyhow!(
                    "unknown boilermates attribute: `{}`",
                    quote::quote!(#attr)
                )),
            })
            .try_collect()
    }
}

fn extract_nested_list(meta_list: &MetaList) -> anyhow::Result<Vec<String>> {
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;

    let nested =
        Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated.parse2(meta_list.tokens.clone())?;

    Ok(nested
        .into_iter()
        .map(|n| n.to_string())
        .collect())
}
