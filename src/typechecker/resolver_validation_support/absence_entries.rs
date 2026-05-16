trait AbsentMetadataValidation<const N: usize>: Copy {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; N];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AbsentMetadataEntry {
    present: bool,
    code: &'static str,
    label: &'static str,
}

impl AbsentMetadataEntry {
    fn new(present: bool, code: &'static str, label: &'static str) -> Self {
        Self {
            present,
            code,
            label,
        }
    }

    fn message(self, symbol_kind: &str, name: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has {} metadata, expected none",
            self.label
        )
    }
}
