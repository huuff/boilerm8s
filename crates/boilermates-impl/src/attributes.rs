use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Attribute, Lit, Meta, MetaList};

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
                                BoilermatesAttrFor::try_from(attr)?,
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

impl TryFrom<MetaList> for BoilermatesAttrFor {
    type Error = anyhow::Error;

    fn try_from(list_attr: MetaList) -> Result<Self, Self::Error> {
        use syn::parse::Parser;
        use syn::punctuated::Punctuated;

        let nested =
            Punctuated::<Lit, syn::Token![,]>::parse_terminated.parse2(list_attr.tokens)?;

        if nested.len() != 2 {
            anyhow::bail!("`#[boilermates(attr_for(...))]` must have two string literal arguments");
        }

        let mut list_iter = nested.into_iter();

        match (
            list_iter.next().expect("we just checked length is 2"),
            list_iter.next().expect("we just checked length is 2"),
        ) {
            (Lit::Str(struct_name), Lit::Str(attr_list)) => Ok(Self {
                target_struct: struct_name.value(),
                attribute: {
                    let token_stream: TokenStream = attr_list
                        .value()
                        .parse::<TokenStream>()
                        .map_err(|_| anyhow::anyhow!("can't parse attr"))?;
                    let q = quote! { #token_stream };
                    parse_quote! { #q }
                },
            }),
            _ => anyhow::bail!(
                "`#[boilermates(attr_for(...))]` must have two string literal arguments"
            ),
        }
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
        Punctuated::<Lit, syn::Token![,]>::parse_terminated.parse2(meta_list.tokens.clone())?;
    nested
        .into_iter()
        .map(|n| match n {
            Lit::Str(lit) => Ok(lit.value()),
            _ => anyhow::bail!("Expected a string literal"),
        })
        .collect()
}
