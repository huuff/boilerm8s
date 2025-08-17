use crate::attributes::BoilermatesFieldAttribute;

#[derive(Clone)]
pub struct BoilermatesField {
    pub definition: syn::Field,
    pub default: bool,
    pub in_structs: Vec<String>,
}

impl BoilermatesField {
    pub fn parse_all(
        syn_fields: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
        all_boilermates_structs: Vec<String>,
        main_boilermates_struct: String,
    ) -> Vec<Self> {
        syn_fields
            .into_iter()
            .map(|mut syn_field| {
                let mut default = false;
                let mut in_structs = all_boilermates_structs.clone();

                for boilermates_attr in
                    BoilermatesFieldAttribute::extract(&mut syn_field.attrs).unwrap()
                {
                    match boilermates_attr {
                        BoilermatesFieldAttribute::OnlyIn(only_in) => in_structs = only_in.0,
                        BoilermatesFieldAttribute::NotIn(not_in) => {
                            in_structs.retain(|strukt| !not_in.0.contains(&strukt))
                        }
                        BoilermatesFieldAttribute::Default => default = true,
                        BoilermatesFieldAttribute::OnlyInSelf => {
                            in_structs = vec![main_boilermates_struct.clone()]
                        }
                    }
                }

                Self {
                    definition: syn_field,
                    default,
                    in_structs,
                }
            })
            .collect()
    }

    pub fn name(&self) -> syn::Ident {
        self.definition
            .ident
            .clone()
            .unwrap_or_else(|| panic!("Can't get field name. This should never happen."))
    }
}
