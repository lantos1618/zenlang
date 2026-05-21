use super::*;
use crate::ast::AstType;

impl SymbolTable {
    #[cfg(test)]
    pub(crate) fn set_parameter_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_count: Option<usize>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_count = parameter_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_type_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_type_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_type_names = parameter_type_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_types_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_types: Option<Vec<AstType>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_types = parameter_types;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_names = parameter_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_return_type_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        return_type_name: Option<String>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.return_type_name = return_type_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_return_type_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        return_type: Option<AstType>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.return_type = return_type;
        }
    }
}
