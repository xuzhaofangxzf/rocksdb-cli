use colored::Colorize;
use rustyline::completion::Pair;
use rustyline::completion::{Completer, FilenameCompleter};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

#[derive(Default, Helper)]
pub struct CliHelper {
    pub commands: Vec<String>,
    pub filename_completer: FilenameCompleter,
}

impl CliHelper {
    pub fn new(commands: Vec<String>) -> Self {
        let filename_completer = FilenameCompleter::new();
        Self {
            commands,
            filename_completer,
        }
    }
}

impl Completer for CliHelper {
    // Define methods here
    type Candidate = Pair;
    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        // 如果行以空格结尾或为空，尝试补全路径
        if line.ends_with(' ') || line.is_empty() {
            return self.filename_completer.complete(line, pos, ctx);
        }
        // 否则尝试补全命令
        let mut candidates = Vec::new();
        for cmd in &self.commands {
            if cmd.starts_with(line) {
                candidates.push(Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                });
            }
        }
        // 如果没有命令匹配，尝试路径补全
        if candidates.is_empty() {
            self.filename_completer.complete(line, pos, ctx)
        } else {
            Ok((0, candidates))
        }
    }
}

impl Hinter for CliHelper {
    type Hint = String;
}

impl Highlighter for CliHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        _ = default;
        std::borrow::Cow::Owned(format!("{}", prompt.bright_green()))
    }
}

impl Validator for CliHelper {}
