extern crate core;

use std::path::{Path, PathBuf};

fn main() {
    let builder = tonic_build::configure()
        .type_attribute("netavark_proxy.Lease", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.DhcpV4Lease", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.DhcpV6Lease", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.IPResponse", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.Ipv4Addr", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.Lease", "#[derive(serde::Deserialize)]")
        .type_attribute(
            "netavark_proxy.DhcpV4Lease",
            "#[derive(serde::Deserialize)]",
        )
        .type_attribute(
            "netavark_proxy.DhcpV6Lease",
            "#[derive(serde::Deserialize)]",
        )
        .type_attribute("netavark_proxy.IPResponse", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.Ipv4Addr", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(Eq)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(Hash)]")
        .type_attribute(
            "netavark_proxy.NetworkConfig",
            "#[derive(serde::Deserialize)]",
        )
        .type_attribute(
            "netavark_proxy.NetworkConfig",
            "#[derive(serde::Serialize)]",
        )
        .out_dir(PathBuf::from("proto-build"));

    builder
        .compile(&[Path::new("proto/proxy.proto")], &[Path::new("proto")])
        .unwrap_or_else(|e| panic!("Failed at builder: {:?}", e.to_string()));
}
