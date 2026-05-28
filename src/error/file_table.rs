pub type FileId = u32;

#[derive(Default)]
pub struct FileTable {
    files: Vec<SourceFile>,
}

struct SourceFile {
    path: String,
    line_starts: Vec<u32>,
}

impl FileTable {
    pub fn add_file(&mut self, path: String, source: &str) -> FileId {
        let line_starts = compute_line_starts(source);
        let id = self.files.len() as FileId;
        self.files.push(SourceFile { path, line_starts });
        id
    }

    pub fn get_path(&self, file_id: FileId) -> Option<&str> {
        self.files.get(file_id as usize).map(|f| f.path.as_str())
    }

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

fn compute_line_starts(source: &str) -> Vec<u32> {
    let mut starts = vec![0u32];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            starts.push((i + 1) as u32);
        }
    }
    starts
}
