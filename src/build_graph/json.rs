use serde::Serialize;

use super::{BuildGraph, BuildTarget, HostEffect};

#[derive(Serialize)]
struct BuildGraphJson<'a> {
    format: &'static str,
    schema_version: u32,
    semantic_status: &'static str,
    targets: &'a [BuildTarget],
    declared_host_effects: &'a [HostEffect],
    used_host_effects: &'a [HostEffect],
}

impl BuildGraph {
    pub fn canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&BuildGraphJson {
            format: "zen.build_graph.v0",
            schema_version: 0,
            semantic_status: "deterministic",
            targets: &self.targets,
            declared_host_effects: &self.declared_host_effects,
            used_host_effects: &self.used_host_effects,
        })
    }
}
