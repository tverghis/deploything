use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct AgentCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Connect to the control plane and begin handling commands.
    Start {
        /// The hostname at which the control plane is located.
        #[arg(short = 'n', long = "hostname", default_value = "localhost")]
        control_plane_hostname: String,
        /// The port number on which the control plane is listening for websocket connections.
        #[arg(short = 'p', long = "port", default_value_t = 4040)]
        control_plane_port: u16,
        /// The interval, in seconds, at which the agent will send snapshots to the control plane.
        #[arg(short = 'i', long = "snapshot-interval", default_value_t = 10)]
        snapshot_interval_secs: u16,
    },
}
