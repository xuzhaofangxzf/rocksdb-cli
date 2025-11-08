use crate::{
    command::DBCommand,
    db::DBHelper,
    display::{print_column_families, print_database_info, print_key_value_list},
    utility::write_output_to_file,
};
use anyhow::Result;
use colored::Colorize;
use rustyrepl::ReplCommandProcessor;
use std::cell::RefCell;

#[derive(Debug)]
pub struct CliProcessor {
    pub db_helper: RefCell<DBHelper>,
}

impl ReplCommandProcessor<DBCommand> for CliProcessor {
    fn is_quit(&self, command: &str) -> bool {
        matches!(command, "quit" | "exit")
    }
    fn process_command(&self, command: DBCommand) -> Result<()> {
        match command {
            DBCommand::List => {
                self.handle_list();
            }
            DBCommand::Use { name } => {
                self.handle_use(name);
            }
            DBCommand::Delete { key } => {
                self.handle_delete(&key)?;
            }

            DBCommand::Get { key, json } => {
                self.db_helper.borrow().get(&key, json)?;
            }
            DBCommand::Keys { limit } => {
                if let Ok(keys) = self.db_helper.borrow().get_keys(limit) {
                    for key in keys {
                        println!("{}", key.bright_green());
                    }
                }
            }
            DBCommand::Info => {
                print_database_info(
                    &self.db_helper.borrow().db,
                    &self.db_helper.borrow().path,
                    &self.db_helper.borrow().current_cf,
                )
                .unwrap();
            }

            DBCommand::Prefix {
                prefix,
                with_highlight,
                limit,
                all,
                output,
            } => {
                if let Ok(key_values) = self.db_helper.borrow().prefix(&prefix, with_highlight) {
                    self.print_or_output_to_file(key_values, all, limit, output.as_deref())?;
                }
            }

            DBCommand::Scan {
                start,
                end,
                reverse,
                limit,
                all,
                output,
            } => {
                if let Ok(key_values) =
                    self.db_helper
                        .borrow()
                        .scan(start.as_deref(), end.as_deref(), reverse)
                {
                    self.print_or_output_to_file(key_values, all, limit, output.as_deref())?;
                }
            }

            DBCommand::ContainsKey { key } => {
                if self.db_helper.borrow().contains_stringkey(&key) {
                    println!("Key {} exists", key.bright_green());
                } else {
                    println!("Key {} doesn't exists", key.bright_red());
                }
            }

            DBCommand::SearchKey {
                key,
                with_highlight,
                limit,
                all,
                output,
            } => {
                if let Ok(key_values) = self.db_helper.borrow().search_key(&key, with_highlight) {
                    self.print_or_output_to_file(key_values, all, limit, output.as_deref())?;
                }
            }

            DBCommand::SearchValue {
                value,
                with_highlight,
                limit,
                all,
                output,
            } => {
                if let Ok(key_values) = self.db_helper.borrow().search_value(&value, with_highlight)
                {
                    self.print_or_output_to_file(key_values, all, limit, output.as_deref())?;
                }
            }
            _ => println!("Unknown command"),
        }
        Ok(())
    }

    fn get_prompt(&self) -> String {
        format!("[{}] >>", self.db_helper.borrow().current_cf.trim())
    }
}

impl CliProcessor {
    pub fn new(db_helper: DBHelper) -> Self {
        Self {
            db_helper: RefCell::new(db_helper),
        }
    }

    fn handle_list(&self) {
        print_column_families(
            &self.db_helper.borrow().cf_list,
            &self.db_helper.borrow().current_cf,
        );
    }

    fn handle_use(&self, name: String) {
        if self.db_helper.borrow().cf_list.contains(&name) {
            self.db_helper.borrow_mut().current_cf = name.clone();
            println!("DB switched to column family {}", name.bright_green());
        } else {
            println!("No column family {} selected", name.bright_red());
        }
    }

    fn handle_delete(&self, key: &str) -> Result<()> {
        self.db_helper.borrow_mut().delete(key)?;
        println!("Key {} deleted", key.bright_green());
        Ok(())
    }

    fn print_or_output_to_file<T: Iterator<Item = (Vec<u8>, Vec<u8>)>>(
        &self,
        key_values: T,
        all: bool,
        limit: usize,
        output: Option<&str>,
    ) -> Result<()> {
        if let Some(out_file) = output {
            if all {
                write_output_to_file(key_values, &out_file)?;
            } else {
                write_output_to_file(key_values.take(limit), &out_file)?;
            }
        } else {
            if all {
                print_key_value_list(key_values);
            } else {
                print_key_value_list(key_values.take(limit));
            }
        }
        Ok(())
    }
}
