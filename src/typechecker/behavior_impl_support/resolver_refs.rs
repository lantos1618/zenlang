use super::*;

impl TypeChecker {
    pub(in crate::typechecker) fn resolver_impl_ref_for(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<BehaviorRefMetadata> {
        self.resolver_behavior_ref_for(
            BehaviorRefRole::Impl,
            type_name,
            behavior,
            behavior_type_args,
        )
    }

    pub(in crate::typechecker) fn resolver_behavior_ref_for(
        &mut self,
        role: BehaviorRefRole,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<BehaviorRefMetadata> {
        match role {
            BehaviorRefRole::Impl => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_impl_refs,
                type_name,
                behavior,
                behavior_type_args,
            ),
            BehaviorRefRole::Required => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_required_refs,
                type_name,
                behavior,
                behavior_type_args,
            ),
            BehaviorRefRole::Parent => None,
        }
    }

    pub(in crate::typechecker) fn behavior_ref_parts<'a>(
        resolver_ref: Option<&'a BehaviorRefMetadata>,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        resolver_ref
            .map(|reference| (reference.name.as_str(), reference.type_args.as_slice()))
            .unwrap_or((behavior, behavior_type_args))
    }

    pub(in crate::typechecker) fn pop_resolver_behavior_ref(
        resolver_backed_collection: bool,
        refs_by_type: &mut HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get_mut(type_name)?;
        Self::pop_resolver_behavior_ref_from_queue(refs, behavior, behavior_type_args)
    }

    pub(in crate::typechecker) fn should_skip_missing_resolver_behavior_ref(
        &self,
        resolver_ref: Option<&BehaviorRefMetadata>,
        type_name: &str,
        missing_refs: &HashSet<String>,
    ) -> bool {
        self.resolver_backed_collection
            && resolver_ref.is_none()
            && missing_refs.contains(type_name)
    }

    pub(in crate::typechecker) fn pop_resolver_behavior_ref_from_queue(
        refs: &mut VecDeque<BehaviorRefMetadata>,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<BehaviorRefMetadata> {
        let index = Self::resolver_behavior_ref_queue_index(refs, behavior, behavior_type_args)?;
        refs.remove(index)
    }

    pub(in crate::typechecker) fn resolver_behavior_impl_ref_for_peek(
        &self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<&BehaviorRefMetadata> {
        Self::peek_resolver_behavior_ref(
            self.resolver_backed_collection,
            &self.resolver_behavior_impl_refs,
            type_name,
            behavior,
            behavior_type_args,
        )
    }

    pub(in crate::typechecker) fn peek_resolver_behavior_ref<'a>(
        resolver_backed_collection: bool,
        refs_by_type: &'a HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<&'a BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get(type_name)?;
        Self::resolver_behavior_ref_queue_index(refs, behavior, behavior_type_args)
            .and_then(|index| refs.get(index))
    }

    pub(in crate::typechecker) fn resolver_behavior_ref_queue_index(
        refs: &VecDeque<BehaviorRefMetadata>,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<usize> {
        refs.iter()
            .position(|reference| {
                reference.name == behavior && reference.type_args == behavior_type_args
            })
            .or_else(|| {
                Self::named_queue_index(refs, behavior, |reference| reference.name.as_str())
            })
    }

    pub(in crate::typechecker) fn named_queue_index<T>(
        items: &VecDeque<T>,
        name: &str,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        items
            .iter()
            .position(|item| item_name(item) == name)
            .or_else(|| (!items.is_empty()).then_some(0))
    }

    pub(in crate::typechecker) fn named_queue_index_preserving_future_front<'a, T>(
        items: &VecDeque<T>,
        name: &str,
        future_names: impl IntoIterator<Item = &'a str>,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        if let Some(index) = items.iter().position(|item| item_name(item) == name) {
            return Some(index);
        }

        let front_name = item_name(items.front()?);
        (!future_names
            .into_iter()
            .any(|future_name| future_name == front_name))
        .then_some(0)
    }

    pub(in crate::typechecker) fn resolver_behavior_impl_ref_parts<'a>(
        &'a self,
        type_name: &str,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        match self.resolver_behavior_impl_ref_for_peek(type_name, behavior, behavior_type_args) {
            Some(implementation) => (
                implementation.name.as_str(),
                implementation.type_args.as_slice(),
            ),
            None => (behavior, behavior_type_args),
        }
    }
}
