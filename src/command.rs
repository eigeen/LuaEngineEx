use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    /// Reload one or all scripts
    Reload {
        /// The name of the script to reload. If not specified, all scripts will be reloaded.
        script: Option<String>,
    },
    /// Debug commands
    Debug {
        #[command(subcommand)]
        command: DebugCommand,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum DebugCommand {
    /// Print all VMs
    Vm,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reload() {
        let inputs = "/lua reload test1.lua"
            .split_whitespace()
            .collect::<Vec<&str>>();
        let cli = Cli::try_parse_from(inputs).unwrap();
        assert_eq!(
            cli.command,
            Command::Reload {
                script: Some("test1.lua".to_string())
            }
        );
    }

    #[test]
    fn test_reload_all() {
        let args = ["/lua", "reload"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.command, Command::Reload { script: None });
    }

    #[test]
    fn test_debug_vm() {
        let inputs = "/lua debug vm".split_whitespace().collect::<Vec<&str>>();
        let cli = Cli::try_parse_from(inputs).unwrap();
        assert_eq!(
            cli.command,
            Command::Debug {
                command: DebugCommand::Vm
            }
        );
    }
}
