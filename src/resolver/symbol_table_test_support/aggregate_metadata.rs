use super::*;
use crate::ast::AstType;

impl SymbolTable {
    #[cfg(test)]
    pub(crate) fn set_field_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_count: Option<usize>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.field_count = field_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_field_type_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_type_names: Option<Vec<(String, String)>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.field_type_names = field_type_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_field_types_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_types: Option<Vec<(String, AstType)>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.field_types = field_types;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.variant_names = variant_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_owner_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_owner_name: Option<String>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.variant_owner_name = variant_owner_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_count: Option<usize>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.variant_payload_count = variant_payload_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_type_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_type_name: Option<String>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.variant_payload_type_name = variant_payload_type_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_type_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_type: Option<AstType>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.variant_payload_type = variant_payload_type;
        }
    }
}
