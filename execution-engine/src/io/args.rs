use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Port to listen for Admin commands (REP socket)
    #[arg(long, default_value_t = 5560)]
    pub admin_port: u16,

    /// Initial Multiplexer port to connect to (PUB socket)
    /// Can be specified multiple times for multiple initial strategies
    #[arg(long)]
    pub multiplexer_ports: Vec<u16>,

    /// Port to receive Market Data (PUB socket from Data Pipeline)
    /// Engine will Connect (SUB) to this port.
    #[arg(long, default_value_t = 5562)]
    pub data_port: u16,

    /// Port to Publish Orders (PUB socket)
    /// Engine will Bind (PUB) to this port.
    #[arg(long, default_value_t = 5563)]
    pub order_port: u16,
}
