extern crate core;

use std::path::{PathBuf};

fn main() {
    let builder = tonic_build::configure()
        .type_attribute("netavark_proxy.Lease", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.DhcpV4Lease", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.DhcpV6Lease", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.IPResponse", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.Ipv4Addr", "#[derive(serde::Serialize)]")
        .type_attribute("netavark_proxy.Lease", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.DhcpV4Lease", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.DhcpV6Lease", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.IPResponse", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.Ipv4Addr", "#[derive(serde::Deserialize)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(Eq)]")
        .type_attribute("netavark_proxy.MacAddress", "#[derive(Hash)]")
        .out_dir(PathBuf::from("src/grpc/"));

    builder
        .compile(&["proto/proxy.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to complie proto {:?}", e));

}