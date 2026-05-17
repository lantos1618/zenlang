/// Index into the FileTable. 0 is reserved for "no file" / test usage.
pub type FileId = u32;

/// Maps FileId -> (path, source text, line_starts cache).
pub struct FileTable {
    files: Vec<SourceFile>,
}

struct SourceFile {
    path: String,
    source: String,
    /// Byte offsets of each line start (computed on insert, cached).
    line_starts: Vec<u32>,
}

impl FileTable {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Add a file and return its FileId.
    pub fn add_file(&mut self, path: String, source: String) -> FileId {
        let line_starts = compute_line_starts(&source);
        let id = self.files.len() as FileId;
        self.files.push(SourceFile {
            path,
            source,
            line_starts,
        });
        id
    }

    pub fn get_source(&self, file_id: FileId) -> Option<&str> {
        self.files.get(file_id as usize).map(|f| f.source.as_str())
    }

    pub fn get_path(&self, file_id: FileId) -> Option<&str> {
        self.files.get(file_id as usize).map(|f| f.path.as_str())
    }

    /// Resolve a byte offset to (line, col), both 0-based.
    /// Returns None if file_id is invalid or offset is out of range.
    pub fn line_col(&self, file_id: FileId, byte_offset: u32) -> Option<(u32, u32)> {
        let file = self.files.get(file_id as usize)?;
        let line = match file.line_starts.binary_search(&byte_offset) {
            Ok(exact) => exact,
            Err(insert) => insert.saturating_sub(1),
        };
        let col = byte_offset - file.line_starts[line];
        Some((line as u32, col))
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

impl Default for FileTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute byte offsets of each line start in `source`.
fn compute_line_starts(source: &str) -> Vec<u32> {
    let mut starts = vec![0u32];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            starts.push((i + 1) as u32);
        }
    }
    starts
}
