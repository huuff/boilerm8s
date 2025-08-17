use heck::ToPascalCase as _;

use crate::model::fields::BoilermatesField;

impl BoilermatesField {
    pub fn trait_name(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("Has{}", &self.name().to_string().to_pascal_case()),
            proc_macro2::Span::call_site(),
        )
    }

    pub fn neg_trait_name(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("HasNo{}", &self.name().to_string().to_pascal_case()),
            proc_macro2::Span::call_site(),
        )
    }
}
