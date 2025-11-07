use clap::Parser;
use rocksdb_cli::cli_helper::CliHelper;
use rocksdb_cli::cli_processor::CliProcessor;
use rocksdb_cli::command::{Cli, InterCli};
use rocksdb_cli::db::DBHelper;
use rustyrepl::{Repl, ReplCommandProcessor};
pub fn main() {
    let cli = Cli::parse();
    let helper = DBHelper::new(&cli.path, cli.readonly);
    let commands = vec![
        "help".into(),
        "list".into(),
        "info".into(),
        "use".into(),
        "keys".into(),
        "contains-key".into(),
        "search-value".into(),
        "search-key".into(),
        "prefix".into(),
        "exit".into(),
        "put".into(),
        "get".into(),
        "delete".into(),
        "scan".into(),
        "quit".into(),
    ];
    let cli_helper = CliHelper::new(commands);
    println!("RocksDB Interactive Shell");
    println!("Type 'help' for available commands");
    let processor: Box<dyn ReplCommandProcessor<InterCli>> = Box::new(CliProcessor::new(helper));
    let mut repl = Repl::<InterCli, CliHelper>::new(
        processor,
        Some("./history_file".to_string()),
        Some(cli_helper),
    )
    .unwrap();

    repl.process().unwrap();
}
