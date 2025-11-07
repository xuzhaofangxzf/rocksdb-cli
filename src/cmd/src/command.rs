use clap::Parser;
use clap::Subcommand;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to RocksDB directory
    #[arg(short, long)]
    pub path: String,
    #[arg(default_value = "true")]
    pub readonly: Option<bool>,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct InterCli {
    #[command(subcommand)]
    pub command: DBCommand,
}

#[derive(Debug, Subcommand)]
pub enum DBCommand {
    /// List all column families
    List,
    /// Get information about the database
    Info,
    /// Switch to a different column family
    Use { name: String },
    /// Get value for a key
    Get {
        key: String,
        #[arg(short, long, default_value_t = false)]
        json: bool,
    },
    ///get all the keys of the current column family
    Keys {
        #[arg(short, long, default_value_t = 10000)]
        limit: usize,
    },

    ContainsKey {
        #[arg(short, long)]
        key: String,
    },

    SearchValue {
        #[arg(short, long)]
        value: String,
        /// Highlight matched keys, use --with-highlight/-w to highlight matched values
        #[arg(short, long, default_value_t = false)]
        with_highlight: bool,
        /// shows limit numbers of keys
        #[arg(short, long, default_value_t = 1000)]
        limit: usize,
        /// search all the values that match the given key without limit
        #[arg(short, long, default_value_t = false)]
        all: bool,
        #[arg(short, long)]
        output: Option<String>,
    },

    SearchKey {
        #[arg(short, long)]
        key: String,
        /// Highlight matched keys, use --with-highlight/-w to highlight matched keys
        #[arg(short, long, default_value_t = false)]
        with_highlight: bool,
        /// shows limit numbers of keys
        #[arg(short, long, default_value_t = 1000)]
        limit: usize,
        /// search all the keys that match the given key without limit
        #[arg(short, long, default_value_t = false)]
        all: bool,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Put a key-value pair
    Put { key: String, value: String },
    /// Delete a key
    Delete { key: String },
    /// Scan key-value pairs
    Scan {
        /// Start key (inclusive)
        #[arg(short, long)]
        start: Option<String>,
        /// End key (exclusive)
        #[arg(short, long)]
        end: Option<String>,
        #[arg(short, long, default_value_t = false)]
        reverse: bool,
        /// Maximum number of keys to return
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
        #[arg(short, long, default_value_t = false)]
        all: bool,
        #[arg(short, long)]
        output: Option<String>,
    },
    Prefix {
        /// Prefix to scan
        #[arg(short, long)]
        prefix: String,
        /// Highlight matched keys, use --with-highlight/-w to highlight matched keys
        #[arg(short, long, default_value_t = false)]
        with_highlight: bool,
        /// Maximum number of keys to return
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
        #[arg(short, long, default_value_t = false)]
        all: bool,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Exit the program
    Exit,
}
