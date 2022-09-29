use clap::{Parser, Subcommand};
use http::Uri;
use log::debug;
use std::process;
use tokio::net::UnixStream;
use tonic::transport::Endpoint;
use tonic::{Code, Status};
use tower::service_fn;

//    ** This client represents the netavark binary which will establish a connection **
use netavark_proxy::commands::{setup, teardown};
use netavark_proxy::g_rpc::netavark_proxy_client::NetavarkProxyClient;
use netavark_proxy::g_rpc::Lease;
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
    let input_config = netavark_proxy::g_rpc::NetworkConfig::load(&file)?;

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

    let client = NetavarkProxyClient::new(channel);

    let result = match opts.subcmd {
        SubCommand::Setup(_) => {
            let s = setup::Setup::new(input_config);
            s.exec(client).await
        }
        SubCommand::Teardown(_) => {
            let t = teardown::Teardown::new(input_config);
            t.exec(client).await
        }
    };

    let r = match result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e.message());
            process_failure(e)
        }
    };

    let pp = ::serde_json::to_string_pretty(&r.into_inner());
    // TODO this should probably return an empty lease so consumers
    // don't soil themselves
    println!("{}", pp.unwrap_or_else(|_| "".to_string()));
    Ok(())
}

//
// process_failure makes the client exit with a specific
// error code
//
fn process_failure(status: Status) -> tonic::Response<Lease> {
    let mut rc: i32 = 1;

    match status.code() {
        Code::Ok => {}
        Code::Cancelled => {}
        Code::Unknown => {
            rc = 155;
        }
        Code::InvalidArgument => {
            rc = 156;
        }
        Code::DeadlineExceeded => {}
        Code::NotFound => {
            rc = 6;
        }
        Code::AlreadyExists => {}
        Code::PermissionDenied => {}
        Code::ResourceExhausted => {}
        Code::FailedPrecondition => {}
        Code::Aborted => {}
        Code::OutOfRange => {}
        Code::Unimplemented => {}
        Code::Internal => {}
        Code::Unavailable => {}
        Code::DataLoss => {}
        Code::Unauthenticated => {}
    }
    process::exit(rc)
}
