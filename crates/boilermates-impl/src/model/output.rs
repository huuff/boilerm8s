use derive_more::{Constructor, IntoIterator};
use indexmap::IndexMap;

use super::fields::BoilermatesField;

nestify::nest! {
  // XXX: Use an indexmap so it's insertion order and deterministic
  #[derive(Default, IntoIterator)]
  pub struct OutputStructs(
    #[into_iterator(owned, ref, ref_mut)]
    #>[derive(Default)]
    pub IndexMap<String, pub struct OutputStruct {
      pub attributes: Vec<syn::Attribute>,
      #>[derive(Clone, Constructor)]
      pub fields: Vec<pub struct OutputField {
        pub definition: syn::Field,
        pub default: bool,
      }>,
  }>)
}

impl OutputStructs {
    /// Returns the names of all the output structs
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|it| it.as_str())
    }

    /// Initializes the output with the main struct and all the mates
    pub fn initialize(
        item: &syn::DeriveInput,
        attrs: proc_macro2::TokenStream,
    ) -> Result<Self, anyhow::Error> {
        use syn::parse::Parser as _;

        let mut output = Self::default();

        output.0.insert(
            item.ident.to_string(),
            OutputStruct {
                fields: Default::default(),
                attributes: item.attrs.clone(),
            },
        );

        for attr in syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated
            .parse2(attrs)
            .unwrap()
        {
            output.0.insert(attr.to_string(), Default::default());
        }

        Ok(output)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut OutputStruct> {
        self.0.get_mut(name)
    }
}

impl OutputStruct {
    pub fn missing_fields_from(&self, other: &Self) -> Vec<OutputField> {
        self.fields.iter().fold(vec![], |mut acc, field| {
            if !other.fields.contains(field) {
                acc.push(field.clone())
            }
            acc
        })
    }

    pub fn same_fields_as(&self, other: &Self) -> Vec<OutputField> {
        self.fields.iter().fold(vec![], |mut acc, field| {
            if other.fields.contains(field) {
                acc.push(field.clone())
            }
            acc
        })
    }
}

impl From<BoilermatesField> for OutputField {
    fn from(value: BoilermatesField) -> Self {
        Self::new(value.definition, value.default)
    }
}

impl OutputField {
    // TODO: Pretty much copy-paste from `BoilermatesField`, DRY?
    pub fn name(&self) -> syn::Ident {
        self.definition
            .ident
            .clone()
            .unwrap_or_else(|| panic!("Can't get field name. This should never happen."))
    }
}

impl PartialEq for OutputField {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl From<syn::Field> for OutputField {
    fn from(field: syn::Field) -> Self {
        Self::new(field, false)
    }
}

impl From<OutputField> for syn::Field {
    fn from(field_config: OutputField) -> Self {
        field_config.definition
    }
}
