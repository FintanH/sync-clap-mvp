use std::{convert::Infallible, str::FromStr, time};

use clap::{Args, Parser, ValueEnum};

pub const DEFAULT_SYNC_TIMEOUT: time::Duration = time::Duration::from_secs(9);

// Looking at the `rad sync` code, these are the possible calls we can make:
//
// rad sync status --rid <RID> --sort-by <field> --verbose
//
// rad sync --fetch --announce --replicas <N> --seed <SEED> --timeout <SECS> --rid <RID> --debug
//
// rad sync --inventory

// OLD OUTPUT:
//
// pub const HELP: Help = Help {
//     name: "sync",
//     description: "Sync repositories to the network",
//     version: env!("RADICLE_VERSION"),
//     usage: r#"
// Usage
//
//     rad sync [--fetch | --announce] [<rid>] [<option>...]
//     rad sync --inventory [<option>...]
//     rad sync status [<rid>] [<option>...]
//
//     By default, the current repository is synchronized both ways.
//     If an <rid> is specified, that repository is synced instead.
//
//     The process begins by fetching changes from connected seeds,
//     followed by announcing local refs to peers, thereby prompting
//     them to fetch from us.
//
//     When `--fetch` is specified, any number of seeds may be given
//     using the `--seed` option, eg. `--seed <nid>@<addr>:<port>`.
//
//     When `--replicas` is specified, the given replication factor will try
//     to be matched. For example, `--replicas 5` will sync with 5 seeds.
//
//     When `--fetch` or `--announce` are specified on their own, this command
//     will only fetch or announce.
//
//     If `--inventory` is specified, the node's inventory is announced to
//     the network. This mode does not take an `<rid>`.
//
// Commands
//
//     status                    Display the sync status of a repository
//
// Options
//
//         --sort-by   <field>   Sort the table by column (options: nid, alias, status)
//     -f, --fetch               Turn on fetching (default: true)
//     -a, --announce            Turn on ref announcing (default: true)
//     -i, --inventory           Turn on inventory announcing (default: false)
//         --timeout   <secs>    How many seconds to wait while syncing
//         --seed      <nid>     Sync with the given node (may be specified multiple times)
//     -r, --replicas  <count>   Sync with a specific number of seeds
//     -v, --verbose             Verbose output
//         --debug               Print debug information afer sync
//         --help                Print help
// "#,
// };

// NEW OUTPUT:
// For `rad sync --help`:
//
// Sync repositories to and from the network
//
// Usage:
//   rad sync [--fetch | --announce] [--rid <rid>] [--timeout <secs>] [--debug] [--seed <nid>]
//   rad sync status [--sort-by <field>]
//   rad sync --inventory

// Commands:
//   status
//   help    Print this message or the help of the given subcommand(s)
//
// Options:
//       --rid <rid>            Repository Identifier to be synchronized
//       --debug                Output debug information, if any
//   -v, --verbose              Out verbose information, if any
//       --fetch                When `--fetch` is specified, any number of seeds may be given using the `--seed` option, eg. `--seed <nid>@<addr>:<port>`
//       --announce             When `--announce` is specified, this command will announce changes to the network. Can be used in tandem with `--fetch` to also fetch beforehand
//       --inventory            If `--inventory` is specified, the node's inventory is announced to the network. This mode ignores the `--rid` argument
//   -r, --replicas <replicas>  Sync with at least N replicas [default: 3]
//       --seed <nid>           Sync with the given list of seeds
//       --timeout <seconds>    How long to wait for syncing to complete [default: 9]
//   -h, --help                 Print help
//   -V, --version              Print version
//
// For `rad sync status --help`:
//
// Display the whether other nodes are synced our out-of-sync with this node's signed references
//
// Usage: rad sync status [--sort-by <field>]
//
// Options:
//       --rid <rid>
//           Repository Identifier to be synchronized
//
//       --sort-by <field>
//           Sort by sync status
//
//           [default: status]
//
//           Possible values:
//           - nid:    Sort by Node ID
//           - alias:  Sort by alias
//           - status: Sort by the sync status (default)
//
//       --debug
//           Output debug information, if any
//
//   -v, --verbose
//           Out verbose information, if any
//
//   -h, --help
//           Print help (see a summary with '-h')

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub enum Operation {
    /// Display the whether other nodes are synced our out-of-sync with this
    /// node's signed references
    #[command(override_usage = "rad sync status [--sort-by <field>]")]
    Status {
        /// Sort by sync status
        #[arg(long, value_name = "field", value_enum, default_value_t)]
        sort_by: SortBy,
    },
}

impl Default for Operation {
    fn default() -> Self {
        Self::Status {
            sort_by: SortBy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum SortBy {
    /// Sort by Node ID
    Nid,
    /// Sort by alias
    Alias,
    /// Sort by the sync status (default)
    #[default]
    Status,
}

impl FromStr for SortBy {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nid" => Ok(Self::Nid),
            "alias" => Ok(Self::Alias),
            "status" => Ok(Self::Status),
            _ => Err("invalid `--sort-by` field"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMode {
    Repo {
        settings: SyncSettings,
        direction: SyncDirection,
    },
    Inventory,
}

impl SyncMode {
    pub fn new(args: SyncModeArgs, settings: Option<SyncSettings>) -> SyncMode {
        if args.inventory {
            SyncMode::Inventory
        } else {
            SyncMode::Repo {
                settings: settings.unwrap_or_default(),
                direction: SyncDirection::from(args.directions),
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Args)]
pub struct SyncModeArgs {
    #[command(flatten)]
    directions: Directions,
    /// If `--inventory` is specified, the node's inventory is announced to the
    /// network. This mode ignores the `--rid` argument.
    #[arg(long)]
    inventory: bool,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Args)]
#[group(required = false, multiple = true, conflicts_with = "inventory")]
pub struct Directions {
    /// When `--fetch` is specified, any number of seeds may be given
    /// using the `--seed` option, eg. `--seed <nid>@<addr>:<port>`.
    #[arg(long)]
    fetch: bool,
    /// When `--announce` is specified, this command will announce changes to
    /// the network. Can be used in tandem with `--fetch` to also fetch
    /// beforehand.
    #[arg(long)]
    announce: bool,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum SyncDirection {
    Fetch,
    Announce,
    #[default]
    Both,
}

impl From<Directions> for SyncDirection {
    fn from(value: Directions) -> Self {
        match (value.fetch, value.announce) {
            (true, true) => SyncDirection::Both,
            (true, false) => SyncDirection::Fetch,
            (false, true) => SyncDirection::Announce,
            // We default to both
            (false, false) => SyncDirection::default(),
        }
    }
}

/// Repository sync settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncSettings {
    /// Sync with at least N replicas.
    pub replicas: usize,
    /// Sync with the given list of seeds.
    pub seeds: Vec<NodeId>,
    /// How long to wait for syncing to complete.
    pub timeout: time::Duration,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            replicas: 3,
            seeds: Vec::new(),
            timeout: DEFAULT_SYNC_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub struct SyncSettingsArgs {
    /// Sync with at least N replicas.
    #[arg(long, short, default_value_t = 3, value_name = "replicas")]
    pub replicas: usize,
    /// Sync with the given list of seeds.
    #[arg(long = "seed", action = clap::ArgAction::Append, value_name = "nid")]
    pub seeds: Vec<NodeId>,
    /// How long to wait for syncing to complete.
    #[arg(long, value_name = "seconds", default_value_t = DEFAULT_SYNC_TIMEOUT.as_secs())]
    pub timeout: u64,
}

impl From<SyncSettingsArgs> for SyncSettings {
    fn from(s: SyncSettingsArgs) -> Self {
        Self {
            replicas: s.replicas,
            seeds: s.seeds,
            timeout: time::Duration::from_secs(s.timeout),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoId(String);

impl FromStr for RepoId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeId(String);

impl FromStr for NodeId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Sync repositories to and from the network
#[derive(Debug, Parser)]
#[command(name = "rad")]
#[command(bin_name = "rad sync")]
#[command(version = "1.0.0")]
#[command(override_usage(
    "
  rad sync [--fetch | --announce] [--rid <rid>] [--timeout <secs>] [--debug] [--seed <nid>]
  rad sync status [--sort-by <field>]
  rad sync --inventory
"
))]
pub struct Options {
    /// Repository Identifier to be synchronized
    #[arg(long, global = true, value_name = "rid")]
    pub rid: Option<RepoId>,
    /// Output debug information, if any
    #[arg(long, global = true)]
    pub debug: bool,
    /// Out verbose information, if any
    #[arg(long, short, global = true)]
    pub verbose: bool,
    #[command(flatten)]
    pub sync: SyncModeArgs,
    #[command(flatten)]
    pub settings: SyncSettingsArgs,
    #[command(subcommand)]
    pub op: Option<Operation>,
}
