// Copyright (c) Sean Lawlor
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::Result;
use log::{debug, error, info, warn};
use rustyline::Helper;
use rustyline::error::ReadlineError;
use rustyline::{Editor, history::DefaultHistory};
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::commands::ReplCommandProcessor;

const DEFAULT_HISTORY_FILE_NAME: &str = ".repl_history";

// #[cfg(feature = "async")]
// macro_rules! get_specific_processing_call {
//     ($self:ident, $cli:expr) => {
//         $self.command_processor.process_command($cli).await?
//     };
// }

/// Represents the REPL interface and processing loop
#[derive(Debug)]
pub struct Repl<C, H>
where
    C: clap::Parser,
    H: Helper,
{
    /// The REPL editor interface for the command prompt
    editor: Editor<H, DefaultHistory>,
    /// The history file
    history: Option<PathBuf>,
    /// User-provided command processor responsible for processing parsed command instructions and
    /// executing on them
    command_processor: Box<dyn ReplCommandProcessor<C>>,

    /// Phantom holder for the command structure enum
    _command_type: PhantomData<C>,
}

impl<C, H> Repl<C, H>
where
    C: clap::Parser,
    H: Helper,
{
    // =================== Private Functions =================== //

    /// Format the history file name to a full path for rustyline
    fn get_history_file_path(history_file_name: Option<String>) -> Option<PathBuf> {
        if let Some(history_file) = &history_file_name {
            let path = Path::new(history_file);

            match (
                path.is_file(),
                path.is_dir(),
                path.is_absolute(),
                path.exists(),
                path.extension(),
                path.components(),
            ) {
                (true, _, _, _, _, _) | (_, _, true, true, Some(_), _) => {
                    // is a file, and either exists on disk or is an absolute path to a file
                    Some(path.to_path_buf())
                }
                (_, true, _, _, _, _) => {
                    // it's a directory that exists, but hasn't specified a file-name (i.e. "~")
                    // append on the default filename, and proceed
                    let mut full_path = path.to_path_buf();
                    full_path.push(DEFAULT_HISTORY_FILE_NAME);
                    Some(full_path)
                }
                (_, _, _, _, Some(_), components) if components.clone().count() == 1 => {
                    // there's some file extension and exactly 1 component to the path,
                    // so it's a file but doesn't exist on disk so we put it in the
                    // home folder (or at least try to)
                    dirs::home_dir().map(|mut home_dir| {
                        home_dir.push(history_file);
                        home_dir
                    })
                }
                _ => None,
            }
        } else {
            debug!("REPL history disabled as no history file provided");
            None
        }
    }

    /// Retrieve the rustyline editor with history loaded (if possible)
    fn get_editor(history: &Option<PathBuf>, helper: Option<H>) -> Result<Editor<H, DefaultHistory>>
    where
        H: Helper,
    {
        let mut rl = Editor::<H, DefaultHistory>::new().unwrap();

        if let Some(history_file) = history {
            match rl.load_history(history_file.as_os_str()) {
                Ok(_) => info!("REPL command history file loaded"),
                Err(err) => warn!("Failed to load REPL command history {}", err),
            }
        }
        rl.set_helper(helper);
        Ok(rl)
    }

    /// Close the history file + save all valid command history (if available)
    fn close_history(&mut self) {
        if let Some(history_path) = &self.history {
            match self.editor.save_history(history_path.as_os_str()) {
                Ok(_) => info!("REPL command history updated"),
                Err(err) => warn!("Failed to safe REPL command history with error '{}'", err),
            }
        }
    }

    // =================== Public API =================== //

    /// Construct a new REPL infterface.
    ///
    /// You can supply the (optional) history file for command history. Utilizing rustyline we can
    /// utilize the history for up & down arrow navigation of past commands. Having the history
    /// file be null will be no history is loaded nor stored
    ///
    /// * `history_file` - The optional command history file. Can be a full path, relative path, directory, or just the end filename to utilize
    /// * `prompt` - The prompt to display to the user to enter input. Defaults to ">>"
    pub fn new(
        command_processor: Box<dyn crate::commands::ReplCommandProcessor<C>>,
        history_file: Option<String>,
        helper: Option<H>,
    ) -> Result<Self>
    where
        H: Helper,
    {
        let history_path = Self::get_history_file_path(history_file);
        let editor = Self::get_editor(&history_path, helper)?;
        Ok(Self {
            editor,
            history: history_path,
            command_processor,
            _command_type: PhantomData,
        })
    }

    // /// Execute the REPL, prompting for user input and processing the results
    // #[cfg(feature = "async")]
    // pub async fn process(&mut self) -> Result<()> {
    //     process_block!(self)
    // }

    /// Execute the REPL, prompting for user input and processing the results
    // #[cfg(not(feature = "async"))]
    pub fn process(&mut self) -> Result<()> {
        self.process_command()
    }

    pub fn process_command(&mut self) -> Result<()> {
        loop {
            let readline = self.editor.readline(&self.command_processor.get_prompt());
            match readline {
                Ok(line) => {
                    let parts = shell_words::split(&line);
                    match parts {
                        Ok(commands) => {
                            let mut command = String::new();
                            if let Some(head) = commands.first() {
                                command = String::from(head);
                            }
                            match command.to_lowercase().as_ref() {
                                "" => {} // Loop, someone hit enter needlessly
                                maybe_quit if self.command_processor.is_quit(maybe_quit) => break, // check for quit/exit
                                _ => {
                                    let mut cmd_parts = vec![&command];
                                    cmd_parts.extend(&commands);
                                    // We're only appending valid commands to the history trail
                                    self.editor.add_history_entry(line.as_str()).unwrap();
                                    match C::try_parse_from(cmd_parts) {
                                        Ok(cli) => {
                                            // Call the underlying processing logic
                                            self.command_processor.process_command(cli)?;
                                        }
                                        Err(clap_err) => match clap::Error::kind(&clap_err) {
                                            clap::error::ErrorKind::DisplayHelp
                                            | clap::error::ErrorKind::DisplayVersion => {
                                                println!("{}", clap_err);
                                            }
                                            error => {
                                                println!("{}", error);
                                            }
                                        },
                                    }
                                }
                            }
                            // let mut command = String::new();
                            // if let Some(head) = commands.first() {
                            //     command = String::from(head);
                            // }
                            // match command.to_lowercase().as_ref() {
                            //     "" => {} // Loop, someone hit enter needlessly
                            //     maybe_quit if self.command_processor.is_quit(maybe_quit) => break, // check for quit/exit
                            //     _ => {
                            //         // We're only appending valid commands to the history trail
                            //         self.editor.add_history_entry(line.as_str()).unwrap();
                            //         let mut cmd_parts = vec![command.as_ref()];
                            //         cmd_parts
                            //             .extend(line.trim().split(' ').collect::<Vec<_>>().iter().copied());
                            //         println!("commands = {:?}", cmd_parts);
                            //         match C::try_parse_from(cmd_parts) {
                            //             Ok(cli) => {
                            //                 // Call the underlying processing logic
                            //                 self.command_processor.process_command(cli)?;
                            //             }
                            //             Err(clap_err) => match clap::Error::kind(&clap_err) {
                            //                 clap::error::ErrorKind::DisplayHelp
                            //                 | clap::error::ErrorKind::DisplayVersion => {
                            //                     println!("{}", clap_err);
                            //                 }
                            //                 error => {
                            //                     println!("{}", error);
                            //                 }
                            //             },
                            //         }
                            //     }
                            // }
                        }
                        Err(err) => {
                            error!("{}", err);
                            continue;
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => break, // CTRL-C
                Err(ReadlineError::Eof) => break,         // CTRL-D
                Err(err) => {
                    error!("Error: {:?}", err);
                    break;
                }
            }
        }
        self.close_history();
        Ok(())
    }

    pub fn set_helper(&mut self, helper: Option<H>) {
        self.editor.set_helper(helper);
    }
}
