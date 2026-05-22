use super::*;

#[test]
fn semantic_overlap_tool_uses_embedding_model_not_lexical_fallback() {
    let script = read("tools/semantic_overlap.py");

    assert!(
        script.contains(r#"DEFAULT_MODEL = "Qwen/Qwen3-Embedding-0.6B""#),
        "semantic overlap audit should default to the local open-source Qwen3 embedding model"
    );
    assert!(
        script.contains("No lexical fallback is provided; this audit uses embeddings only."),
        "semantic overlap audit should stay embedding-only"
    );
    assert!(
        !script.contains("TfidfVectorizer") && !script.contains("sklearn"),
        "semantic overlap audit should not regress to TF-IDF or sklearn lexical similarity"
    );
}

#[test]
fn semantic_overlap_outputs_and_model_caches_stay_ignored() {
    let script = read("tools/semantic_overlap.py");
    let gitignore = read(".gitignore");

    for expected in [
        r#"DEFAULT_CACHE_DIR = ".cache/huggingface""#,
        r#"default="target/tmp/semantic-overlap-report.md""#,
        r#"default="target/tmp/semantic-overlap-report.json""#,
    ] {
        assert!(
            script.contains(expected),
            "semantic overlap tool should keep local caches and reports in ignored paths: {expected}"
        );
    }

    for expected in [
        "/target/",
        "/.cache/huggingface/",
        "/.cache/torch/",
        "/hf-cache/",
        "/model-cache/",
        "/models/",
    ] {
        assert!(
            gitignore.contains(expected),
            "semantic overlap artifacts should be ignored by git: {expected}"
        );
    }
}
