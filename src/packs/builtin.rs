use crate::error::Result;
use crate::packs::loader::load_manifest_from_str;
use crate::packs::manifest::PackManifest;

pub struct BuiltinPack {
    pub name: &'static str,
    pub yaml_content: &'static str,
}

pub const BUILTIN_PACKS: &[BuiltinPack] = &[
    BuiltinPack {
        name: "aws-cloudwatch",
        yaml_content: include_str!("../../packs/aws-cloudwatch/pack.yaml"),
    },
    BuiltinPack {
        name: "azure-monitor",
        yaml_content: include_str!("../../packs/azure-monitor/pack.yaml"),
    },
    BuiltinPack {
        name: "checkmk",
        yaml_content: include_str!("../../packs/checkmk/pack.yaml"),
    },
    BuiltinPack {
        name: "ci-canary",
        yaml_content: include_str!("../../packs/ci-canary/pack.yaml"),
    },
    BuiltinPack {
        name: "docker",
        yaml_content: include_str!("../../packs/docker/pack.yaml"),
    },
    BuiltinPack {
        name: "example",
        yaml_content: include_str!("../../packs/example/pack.yaml"),
    },
    BuiltinPack {
        name: "gcp-cloud-ops",
        yaml_content: include_str!("../../packs/gcp-cloud-ops/pack.yaml"),
    },
    BuiltinPack {
        name: "kubernetes",
        yaml_content: include_str!("../../packs/kubernetes/pack.yaml"),
    },
    BuiltinPack {
        name: "linux",
        yaml_content: include_str!("../../packs/linux/pack.yaml"),
    },
    BuiltinPack {
        name: "mysql",
        yaml_content: include_str!("../../packs/mysql/pack.yaml"),
    },
    BuiltinPack {
        name: "nginx",
        yaml_content: include_str!("../../packs/nginx/pack.yaml"),
    },
    BuiltinPack {
        name: "terraform-plan",
        yaml_content: include_str!("../../packs/terraform-plan/pack.yaml"),
    },
];

/// Retrieves and deserializes a compiled-in pack by name if it exists.
pub fn get_builtin_pack(name: &str) -> Option<Result<PackManifest>> {
    BUILTIN_PACKS
        .iter()
        .find(|p| p.name == name)
        .map(|p| load_manifest_from_str(p.yaml_content, &format!("<embedded:{}>", p.name)))
}
