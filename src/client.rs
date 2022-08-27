use clap::{Parser, Subcommand};
use log::debug;

//    ** This client represents the netavark binary which will establish a connection **
use netavark_proxy::commands::{setup, teardown};
use netavark_proxy::g_rpc::netavark_proxy_client::NetavarkProxyClient;

#[derive(Parser, Debug)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Use specific grpc_port
    #[clap(short, long)]
    port: Option<String>,
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

pub const XDGRUNTIME: &str = "/run/user/1000/nv-leases";
#[tokio::main]
#[allow(unused)]
// This client assumes you use the default lease directory
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This should be moved to somewhere central.  We also need to add override logic.
    let default_uri = String::from("http://[::1]:10000");

    env_logger::builder().format_timestamp(None).init();
    let opts = Opts::parse();
    let file = opts.file.unwrap_or_else(|| String::from("/dev/stdin"));
    let grpc_port = opts.port.unwrap_or(default_uri);

    debug!("using grpc port: {}", grpc_port);

    let input_config = netavark_proxy::g_rpc::NetworkConfig::load(&file)?;
    println!("{:?}", ::serde_json::to_string_pretty(&input_config));

    let mut client = NetavarkProxyClient::connect(grpc_port).await?;

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
