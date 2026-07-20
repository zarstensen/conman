use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone)]
enum CliAction {
    Push,
    Drop
}

#[derive(Parser)]
struct CliArgs {
    command: CliAction,
    containers: Vec<String>,
    #[arg(short, long = "targets", num_args = 1..)]
    targets: Vec<String>
}
