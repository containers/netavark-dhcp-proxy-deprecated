use clap::{Parser, Subcommand};
use http::Uri;
use log::debug;
use tokio::net::UnixStream;
use tonic::transport::Endpoint;
use tower::service_fn;
//    ** This client represents the netavark binary which will establish a connection **
use netavark_proxy::commands::{setup, teardown};
use netavark_proxy::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use netavark_proxy::{DEFAULT_NETWORK_CONFIG, DEFAULT_UDS_PATH};

#[derive(Parser, Debug)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Use specific uds path
    #[clap(short, long)]
    uds: Option<String>,
    /// Instead of reading from STDIN, read the configuration to be applied from the given file.
    #[clap(short, long)]
    file: Option<String>,
    /// Netavark trig command
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Configures the given network namespace with the given configuration.
    Setup(setup::Setup),
    /// Undo any configuration applied via setup command.
    Teardown(teardown::Teardown),
    // Display info about netavark.
    // Version(version::Version),
}

#[cfg(unix)]
#[tokio::main]
// This client assumes you use the default lease directory
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This should be moved to somewhere central.  We also need to add override logic.

    env_logger::builder().format_timestamp(None).init();
    let opts = Opts::parse();
    let file = opts
        .file
        .unwrap_or_else(|| DEFAULT_NETWORK_CONFIG.to_string());
    let uds_path = opts.uds.unwrap_or_else(|| DEFAULT_UDS_PATH.to_string());

    // We will ignore this uri because uds do not use it
    // if your connector does use the uri it will be provided
    // as the request to the `MakeConnection`.
    let channel = Endpoint::try_from("http://[::1]:10000")?
        .connect_with_connector(service_fn(move |_: Uri| {
            // Connect to a Uds socket
            let path = uds_path.clone();
            debug!("using uds path: {}", &path);
            UnixStream::connect(path)
        }))
        .await?;

    let input_config = netavark_proxy::g_rpc::NetworkConfig::load(&file)?;
    println!("{:?}", ::serde_json::to_string_pretty(&input_config));

    let client = NetavarkProxyClient::new(channel);

    let result = match opts.subcmd {
        SubCommand::Setup(_) => {
            let s = setup::Setup::new(input_config);
            s.exec(client).await?
        }
        SubCommand::Teardown(_) => {
            let t = teardown::Teardown::new(input_config);
            t.exec(client).await?
        }
    };

    println!("{:#?}", ::serde_json::to_string_pretty(&result));
    Ok(())
}
