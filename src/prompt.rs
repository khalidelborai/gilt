//! Interactive prompt module for styled user input with validation, choices, and defaults.
//!
//! Port of Python's rich/prompt.py. Provides `Prompt` for string input,
//! `confirm()` for yes/no questions, `ask_int()` for integer input, and
//! `ask_float()` for float input.

use std::io::{self, BufRead, Write as IoWrite};

use crate::console::Console;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Rustyline completer (feature-gated)
// ---------------------------------------------------------------------------

/// A simple completer that matches from a list of candidate strings.
#[cfg(feature = "readline")]
#[derive(Clone)]
struct ListCompleter {
    candidates: Vec<String>,
}

#[cfg(feature = "readline")]
impl rustyline::completion::Completer for ListCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<String>)> {
        let prefix = &line[..pos];
        let matches: Vec<String> = self
            .candidates
            .iter()
            .filter(|c| c.starts_with(prefix))
            .cloned()
            .collect();
        Ok((0, matches))
    }
}

#[cfg(feature = "readline")]
impl rustyline::hint::Hinter for ListCompleter {
    type Hint = String;
}

#[cfg(feature = "readline")]
impl rustyline::highlight::Highlighter for ListCompleter {}

#[cfg(feature = "readline")]
impl rustyline::validate::Validator for ListCompleter {}

#[cfg(feature = "readline")]
impl rustyline::Helper for ListCompleter {}

// ---------------------------------------------------------------------------
// InvalidResponse
// ---------------------------------------------------------------------------

/// Error indicating an invalid response from the user.
#[derive(Debug, PartialEq)]
pub struct InvalidResponse {
    /// Human-readable description of why the response was invalid.
    pub message: String,
}

impl std::fmt::Display for InvalidResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for InvalidResponse {}

// ---------------------------------------------------------------------------
// Prompt
// ---------------------------------------------------------------------------

/// A styled interactive prompt for user input with validation, choices, and defaults.
///
/// # Examples
///
/// ```no_run
/// use gilt::prompt::Prompt;
///
/// let name = Prompt::new("Enter your name").ask();
/// let fruit = Prompt::new("Pick a fruit")
///     .with_choices(vec!["apple".into(), "orange".into(), "pear".into()])
///     .ask();
/// ```
pub struct Prompt {
    /// The prompt text (parsed from markup).
    pub prompt_text: Text,
    /// Whether to hide input (password mode).
    pub password: bool,
    /// Optional list of valid choices.
    pub choices: Option<Vec<String>>,
    /// Whether choice matching is case-sensitive.
    pub case_sensitive: bool,
    /// Whether to display the default value in the prompt.
    pub show_default: bool,
    /// Whether to display the available choices in the prompt.
    pub show_choices: bool,
    /// Optional default value returned when the user enters empty input.
    pub default: Option<String>,
    /// Optional list of tab-completion candidates.
    ///
    /// When the `readline` feature is enabled and this is `Some`, the prompt
    /// will use `rustyline` to provide interactive tab-completion from the
    /// given list. When the feature is not enabled, this field is ignored and
    /// input is read from standard input as usual.
    pub completions: Option<Vec<String>>,
    /// The console used for rendering prompt text.
    console: Console,
}

impl Prompt {
    /// Create a new prompt with the given text.
    ///
    /// The prompt string is parsed as Rich markup.
    pub fn new(prompt: &str) -> Self {
        let prompt_text = crate::markup::render(prompt, Style::null())
            .unwrap_or_else(|_| Text::new(prompt, Style::null()));
        Prompt {
            prompt_text,
            password: false,
            choices: None,
            case_sensitive: true,
            show_default: true,
            show_choices: true,
            default: None,
            completions: None,
            console: Console::new(),
        }
    }

    /// Set the console for this prompt.
    #[must_use]
    pub fn with_console(mut self, console: Console) -> Self {
        self.console = console;
        self
    }

    /// Set whether the prompt hides input (password mode).
    #[must_use]
    pub fn with_password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Set the list of valid choices.
    #[must_use]
    pub fn with_choices(mut self, choices: Vec<String>) -> Self {
        self.choices = Some(choices);
        self
    }

    /// Set the default value.
    #[must_use]
    pub fn with_default(mut self, default: &str) -> Self {
        self.default = Some(default.to_string());
        self
    }

    /// Set whether choice matching is case-sensitive.
    #[must_use]
    pub fn with_case_sensitive(mut self, case: bool) -> Self {
        self.case_sensitive = case;
        self
    }

    /// Set whether to display the default value in the prompt.
    #[must_use]
    pub fn with_show_default(mut self, show: bool) -> Self {
        self.show_default = show;
        self
    }

    /// Set whether to display the available choices in the prompt.
    #[must_use]
    pub fn with_show_choices(mut self, show: bool) -> Self {
        self.show_choices = show;
        self
    }

    /// Set the list of tab-completion candidates.
    ///
    /// When the `readline` feature is enabled, the prompt will use `rustyline`
    /// to offer interactive tab-completion from the given list. When the
    /// feature is not enabled, this setting is silently ignored.
    #[must_use]
    pub fn with_completions(mut self, completions: Vec<String>) -> Self {
        self.completions = Some(completions);
        self
    }

    /// Build the prompt `Text` including choices and default annotations.
    ///
    /// Format: `"prompt [choice1/choice2/...] (default): "`
    pub fn make_prompt(&self) -> Text {
        let mut prompt = self.prompt_text.clone();
        prompt.end = String::new();

        if self.show_choices {
            if let Some(ref choices) = self.choices {
                let choices_str = format!("[{}]", choices.join("/"));
                let choices_style = Style::parse("magenta bold").unwrap_or_else(|_| Style::null());
                prompt.append_str(" ", None);
                prompt.append_str(&choices_str, Some(choices_style));
            }
        }

        if self.show_default {
            if let Some(ref default) = self.default {
                let default_str = format!("({})", default);
                let default_style = Style::parse("cyan bold").unwrap_or_else(|_| Style::null());
                prompt.append_str(" ", None);
                prompt.append_str(&default_str, Some(default_style));
            }
        }

        prompt.append_str(": ", None);

        prompt
    }

    /// Check whether a value is a valid choice.
    fn check_choice(&self, value: &str) -> bool {
        match &self.choices {
            None => true,
            Some(choices) => {
                let trimmed = value.trim();
                if self.case_sensitive {
                    choices.iter().any(|c| c == trimmed)
                } else {
                    let lower = trimmed.to_lowercase();
                    choices.iter().any(|c| c.to_lowercase() == lower)
                }
            }
        }
    }

    /// Given a validated value, return the canonical form from the choices list.
    ///
    /// For case-insensitive matching, returns the original-cased choice.
    fn resolve_choice(&self, value: &str) -> String {
        let trimmed = value.trim();
        match &self.choices {
            None => trimmed.to_string(),
            Some(choices) => {
                if self.case_sensitive {
                    trimmed.to_string()
                } else {
                    let lower = trimmed.to_lowercase();
                    choices
                        .iter()
                        .find(|c| c.to_lowercase() == lower)
                        .cloned()
                        .unwrap_or_else(|| trimmed.to_string())
                }
            }
        }
    }

    /// Read user input from the provided reader, printing the prompt to stdout.
    ///
    /// This method is the testable core of `ask()`. Tests can inject mock input
    /// via `std::io::Cursor`.
    pub fn ask_with_input<R: BufRead>(&self, input: &mut R) -> String {
        loop {
            let prompt = self.make_prompt();
            let prompt_str = prompt.plain().to_string();
            print!("{}", prompt_str);
            let _ = io::stdout().flush();

            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) => {
                    // EOF: if there's a default, return it; otherwise keep the empty string
                    if let Some(ref default) = self.default {
                        return default.clone();
                    }
                    return String::new();
                }
                Ok(_) => {}
                Err(_) => {
                    if let Some(ref default) = self.default {
                        return default.clone();
                    }
                    return String::new();
                }
            }

            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
            let value = trimmed.to_string();

            // Empty input: return default if available
            if value.trim().is_empty() {
                if let Some(ref default) = self.default {
                    return default.clone();
                }
            }

            // Validate against choices
            if self.choices.is_some() {
                if !self.check_choice(&value) {
                    // Invalid choice — print error and loop
                    eprintln!("Please select one of the available options");
                    continue;
                }
                return self.resolve_choice(&value);
            }

            return value;
        }
    }

    /// Ask the user for input, reading from standard input.
    ///
    /// This is the primary public API. It loops until valid input is received.
    /// When password mode is enabled, terminal echo is disabled so the input
    /// is not visible on screen. When the `readline` feature is enabled and
    /// [`completions`](Prompt::completions) is set, the prompt uses `rustyline`
    /// to provide interactive tab-completion.
    pub fn ask(&self) -> String {
        #[cfg(feature = "interactive")]
        if self.password {
            return self.ask_password();
        }
        #[cfg(not(feature = "interactive"))]
        if self.password {
            // Fall back to regular input when rpassword is unavailable.
            // WARNING: input will be visible on screen.
            eprintln!(
                "warning: gilt built without `interactive` feature; password input will be visible"
            );
        }

        #[cfg(feature = "readline")]
        if self.completions.is_some() {
            return self.ask_readline();
        }

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.ask_with_input(&mut handle)
    }

    /// Readline-based input loop with tab-completion.
    #[cfg(feature = "readline")]
    fn ask_readline(&self) -> String {
        let candidates = self.completions.clone().unwrap_or_default();
        let helper = ListCompleter { candidates };
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut editor = rustyline::Editor::with_config(config).expect("Failed to create editor");
        editor.set_helper(Some(helper));

        loop {
            let prompt = self.make_prompt();
            let prompt_str = prompt.plain().to_string();

            match editor.readline(&prompt_str) {
                Ok(line) => {
                    let value = line
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string();

                    // Empty input: return default if available
                    if value.trim().is_empty() {
                        if let Some(ref default) = self.default {
                            return default.clone();
                        }
                    }

                    // Validate against choices
                    if self.choices.is_some() {
                        if !self.check_choice(&value) {
                            eprintln!("Please select one of the available options");
                            continue;
                        }
                        return self.resolve_choice(&value);
                    }

                    return value;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    if let Some(ref default) = self.default {
                        return default.clone();
                    }
                    return String::new();
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    return String::new();
                }
                Err(_) => {
                    if let Some(ref default) = self.default {
                        return default.clone();
                    }
                    return String::new();
                }
            }
        }
    }

    /// Password input loop — reads without terminal echo using `rpassword`.
    #[cfg(feature = "interactive")]
    fn ask_password(&self) -> String {
        loop {
            let prompt = self.make_prompt();
            let prompt_str = prompt.plain().to_string();
            print!("{}", prompt_str);
            let _ = io::stdout().flush();

            let value = match rpassword::read_password() {
                Ok(v) => v,
                Err(_) => {
                    if let Some(ref default) = self.default {
                        return default.clone();
                    }
                    return String::new();
                }
            };

            // Empty input: return default if available
            if value.trim().is_empty() {
                if let Some(ref default) = self.default {
                    return default.clone();
                }
            }

            // Validate against choices
            if self.choices.is_some() {
                if !self.check_choice(&value) {
                    eprintln!("Please select one of the available options");
                    continue;
                }
                return self.resolve_choice(&value);
            }

            return value;
        }
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

/// Ask a yes/no confirmation question and return a boolean.
///
/// Returns `true` for "y"/"yes", `false` for "n"/"no" (case-insensitive).
/// Loops until valid input is received.
pub fn confirm(prompt: &str) -> bool {
    confirm_with_input(prompt, &mut io::stdin().lock())
}

/// Testable version of `confirm()` that reads from a provided input source.
pub fn confirm_with_input<R: BufRead>(prompt: &str, input: &mut R) -> bool {
    let p = Prompt::new(prompt)
        .with_choices(vec!["y".into(), "n".into()])
        .with_case_sensitive(false)
        .with_show_choices(true);

    loop {
        let prompt_text = p.make_prompt();
        let prompt_str = prompt_text.plain().to_string();
        print!("{}", prompt_str);
        let _ = io::stdout().flush();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => return false,
            Ok(_) => {}
            Err(_) => return false,
        }

        let value = line.trim().to_lowercase();
        match value.as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                eprintln!("Please enter Y or N");
                continue;
            }
        }
    }
}

/// Ask the user for an integer value. Loops until valid input is received.
pub fn ask_int(prompt: &str) -> i64 {
    ask_int_with_input(prompt, &mut io::stdin().lock())
}

/// Testable version of `ask_int()` that reads from a provided input source.
pub fn ask_int_with_input<R: BufRead>(prompt: &str, input: &mut R) -> i64 {
    loop {
        let prompt_text = Prompt::new(prompt).make_prompt();
        let prompt_str = prompt_text.plain().to_string();
        print!("{}", prompt_str);
        let _ = io::stdout().flush();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => {
                eprintln!("Please enter a valid integer number");
                continue;
            }
            Ok(_) => {}
            Err(_) => {
                eprintln!("Please enter a valid integer number");
                continue;
            }
        }

        match line.trim().parse::<i64>() {
            Ok(v) => return v,
            Err(_) => {
                eprintln!("Please enter a valid integer number");
                continue;
            }
        }
    }
}

/// Ask the user for a float value. Loops until valid input is received.
pub fn ask_float(prompt: &str) -> f64 {
    ask_float_with_input(prompt, &mut io::stdin().lock())
}

/// Testable version of `ask_float()` that reads from a provided input source.
pub fn ask_float_with_input<R: BufRead>(prompt: &str, input: &mut R) -> f64 {
    loop {
        let prompt_text = Prompt::new(prompt).make_prompt();
        let prompt_str = prompt_text.plain().to_string();
        print!("{}", prompt_str);
        let _ = io::stdout().flush();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => {
                eprintln!("Please enter a valid number");
                continue;
            }
            Ok(_) => {}
            Err(_) => {
                eprintln!("Please enter a valid number");
                continue;
            }
        }

        match line.trim().parse::<f64>() {
            Ok(v) => return v,
            Err(_) => {
                eprintln!("Please enter a valid number");
                continue;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

/// A prompt that lets users select one option from a numbered list.
///
/// Displays choices as a numbered list and asks the user to enter a number.
///
/// # Examples
///
/// ```no_run
/// use gilt::prompt::Select;
/// use gilt::console::Console;
///
/// let mut console = Console::new();
/// let index = Select::new("Select a color", vec!["Red".into(), "Green".into(), "Blue".into()])
///     .ask(&mut console)
///     .unwrap();
/// ```
pub struct Select {
    /// The prompt text.
    pub prompt: String,
    /// The list of choices to display.
    pub choices: Vec<String>,
    /// Optional 0-indexed default choice.
    pub default: Option<usize>,
    /// Style for the prompt question mark and text.
    pub style: Style,
    /// Style for the choice numbers.
    pub highlight_style: Style,
}

impl Select {
    /// Create a new Select prompt with the given prompt text and choices.
    pub fn new(prompt: &str, choices: Vec<String>) -> Self {
        Select {
            prompt: prompt.to_string(),
            choices,
            default: None,
            style: Style::parse("bold").unwrap_or_else(|_| Style::null()),
            highlight_style: Style::parse("cyan bold").unwrap_or_else(|_| Style::null()),
        }
    }

    /// Set the default choice index (0-based).
    #[must_use]
    pub fn with_default(mut self, index: usize) -> Self {
        self.default = Some(index);
        self
    }

    /// Set the style for the prompt text.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the style for the choice numbers.
    #[must_use]
    pub fn with_highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Format the choice list as a string for display.
    ///
    /// Returns lines like:
    /// ```text
    /// ? Select a color:
    ///   1) Red
    ///   2) Green
    ///   3) Blue
    /// ```
    pub fn format_choices(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("? {}:\n", self.prompt));
        for (i, choice) in self.choices.iter().enumerate() {
            output.push_str(&format!("  {}) {}\n", i + 1, choice));
        }
        output
    }

    /// Format the input prompt line (e.g. "Enter choice [1-3]: " or "Enter choice [1-3] (2): ").
    pub fn format_input_prompt(&self) -> String {
        let n = self.choices.len();
        let mut prompt = format!("Enter choice [1-{}]", n);
        if let Some(default) = self.default {
            prompt.push_str(&format!(" ({})", default + 1));
        }
        prompt.push_str(": ");
        prompt
    }

    /// Parse and validate a single-number input string.
    ///
    /// Returns `Ok(index)` with a 0-based index, or `Err(InvalidResponse)` on invalid input.
    pub fn parse_input(&self, input: &str) -> Result<usize, InvalidResponse> {
        let trimmed = input.trim();

        // Empty input with default
        if trimmed.is_empty() {
            if let Some(default) = self.default {
                if default < self.choices.len() {
                    return Ok(default);
                }
                return Err(InvalidResponse {
                    message: format!(
                        "Default index {} is out of range (1-{})",
                        default + 1,
                        self.choices.len()
                    ),
                });
            }
            return Err(InvalidResponse {
                message: "Please enter a number".to_string(),
            });
        }

        // Parse number
        let num: usize = trimmed.parse().map_err(|_| InvalidResponse {
            message: format!("'{}' is not a valid number", trimmed),
        })?;

        // Validate range (user enters 1-based)
        if num < 1 || num > self.choices.len() {
            return Err(InvalidResponse {
                message: format!("Please enter a number between 1 and {}", self.choices.len()),
            });
        }

        Ok(num - 1) // Convert to 0-based
    }

    /// Show the prompt and return the selected index (0-based).
    ///
    /// Returns an error if choices is empty.
    pub fn ask(&self, console: &mut Console) -> Result<usize, InvalidResponse> {
        if self.choices.is_empty() {
            return Err(InvalidResponse {
                message: "No choices provided".to_string(),
            });
        }
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.ask_with_input(console, &mut handle)
    }

    /// Testable version of `ask()` that reads from a provided input source.
    pub fn ask_with_input<R: BufRead>(
        &self,
        console: &mut Console,
        input: &mut R,
    ) -> Result<usize, InvalidResponse> {
        if self.choices.is_empty() {
            return Err(InvalidResponse {
                message: "No choices provided".to_string(),
            });
        }

        // Print the choice list
        let choices_display = self.format_choices();
        console.print_text(&choices_display);

        loop {
            let prompt_line = self.format_input_prompt();
            print!("{}", prompt_line);
            let _ = io::stdout().flush();

            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) => {
                    // EOF
                    if let Some(default) = self.default {
                        if default < self.choices.len() {
                            return Ok(default);
                        }
                    }
                    return Err(InvalidResponse {
                        message: "No input provided".to_string(),
                    });
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(InvalidResponse {
                        message: format!("Input error: {}", e),
                    });
                }
            }

            match self.parse_input(&line) {
                Ok(index) => return Ok(index),
                Err(msg) => {
                    eprintln!("{}", msg);
                    continue;
                }
            }
        }
    }

    /// Show the prompt and return the selected value.
    pub fn ask_value(&self, console: &mut Console) -> Result<String, InvalidResponse> {
        let index = self.ask(console)?;
        Ok(self.choices[index].clone())
    }

    /// Testable version of `ask_value()` that reads from a provided input source.
    pub fn ask_value_with_input<R: BufRead>(
        &self,
        console: &mut Console,
        input: &mut R,
    ) -> Result<String, InvalidResponse> {
        let index = self.ask_with_input(console, input)?;
        Ok(self.choices[index].clone())
    }
}

// ---------------------------------------------------------------------------
// MultiSelect
// ---------------------------------------------------------------------------

/// A prompt that lets users select multiple options from a numbered list.
///
/// Displays choices as a numbered list and asks the user to enter
/// comma-separated numbers. Also supports "all" to select everything.
///
/// # Examples
///
/// ```no_run
/// use gilt::prompt::MultiSelect;
/// use gilt::console::Console;
///
/// let mut console = Console::new();
/// let indices = MultiSelect::new("Select colors", vec!["Red".into(), "Green".into(), "Blue".into()])
///     .with_min(1)
///     .ask(&mut console)
///     .unwrap();
/// ```
pub struct MultiSelect {
    /// The prompt text.
    pub prompt: String,
    /// The list of choices to display.
    pub choices: Vec<String>,
    /// Pre-selected indices (0-based).
    pub defaults: Vec<usize>,
    /// Minimum number of selections required.
    pub min_selections: usize,
    /// Maximum number of selections allowed (None = unlimited).
    pub max_selections: Option<usize>,
    /// Style for the prompt question mark and text.
    pub style: Style,
    /// Style for the choice numbers.
    pub highlight_style: Style,
}

impl MultiSelect {
    /// Create a new MultiSelect prompt with the given prompt text and choices.
    pub fn new(prompt: &str, choices: Vec<String>) -> Self {
        MultiSelect {
            prompt: prompt.to_string(),
            choices,
            defaults: Vec::new(),
            min_selections: 0,
            max_selections: None,
            style: Style::parse("bold").unwrap_or_else(|_| Style::null()),
            highlight_style: Style::parse("cyan bold").unwrap_or_else(|_| Style::null()),
        }
    }

    /// Set the pre-selected default indices (0-based).
    #[must_use]
    pub fn with_defaults(mut self, indices: Vec<usize>) -> Self {
        self.defaults = indices;
        self
    }

    /// Set the minimum number of selections required.
    #[must_use]
    pub fn with_min(mut self, min: usize) -> Self {
        self.min_selections = min;
        self
    }

    /// Set the maximum number of selections allowed.
    #[must_use]
    pub fn with_max(mut self, max: usize) -> Self {
        self.max_selections = Some(max);
        self
    }

    /// Set the style for the prompt text.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the style for the choice numbers.
    #[must_use]
    pub fn with_highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Format the choice list as a string for display.
    ///
    /// Returns lines like:
    /// ```text
    /// ? Select colors (comma-separated):
    ///   1) Red
    ///   2) Green
    ///   3) Blue
    /// ```
    pub fn format_choices(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("? {} (comma-separated):\n", self.prompt));
        for (i, choice) in self.choices.iter().enumerate() {
            output.push_str(&format!("  {}) {}\n", i + 1, choice));
        }
        output
    }

    /// Format the input prompt line.
    pub fn format_input_prompt(&self) -> String {
        let n = self.choices.len();
        let mut prompt = format!("Enter choices [1-{}, e.g. 1,3]", n);
        if !self.defaults.is_empty() {
            let defaults_str: Vec<String> =
                self.defaults.iter().map(|d| (d + 1).to_string()).collect();
            prompt.push_str(&format!(" ({})", defaults_str.join(",")));
        }
        prompt.push_str(": ");
        prompt
    }

    /// Parse and validate a comma-separated input string.
    ///
    /// Supports individual numbers, comma-separated numbers, and "all".
    /// Returns `Ok(indices)` with 0-based indices, or `Err(InvalidResponse)` on invalid input.
    pub fn parse_input(&self, input: &str) -> Result<Vec<usize>, InvalidResponse> {
        let trimmed = input.trim();

        // Empty input with defaults
        if trimmed.is_empty() {
            if !self.defaults.is_empty() {
                // Validate defaults are in range
                for &d in &self.defaults {
                    if d >= self.choices.len() {
                        return Err(InvalidResponse {
                            message: format!(
                                "Default index {} is out of range (1-{})",
                                d + 1,
                                self.choices.len()
                            ),
                        });
                    }
                }
                return self.validate_count(&self.defaults);
            }
            // Empty with no defaults: return empty set (if min allows it)
            return self.validate_count(&[]);
        }

        // "all" keyword
        if trimmed.eq_ignore_ascii_case("all") {
            let all: Vec<usize> = (0..self.choices.len()).collect();
            return self.validate_count(&all);
        }

        // Parse comma-separated numbers
        let mut indices = Vec::new();
        for part in trimmed.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let num: usize = part.parse().map_err(|_| InvalidResponse {
                message: format!("'{}' is not a valid number", part),
            })?;
            if num < 1 || num > self.choices.len() {
                return Err(InvalidResponse {
                    message: format!("Number {} is out of range (1-{})", num, self.choices.len()),
                });
            }
            let index = num - 1;
            if !indices.contains(&index) {
                indices.push(index);
            }
        }

        self.validate_count(&indices)
    }

    /// Validate selection count against min/max constraints.
    fn validate_count(&self, indices: &[usize]) -> Result<Vec<usize>, InvalidResponse> {
        if indices.len() < self.min_selections {
            return Err(InvalidResponse {
                message: format!(
                    "Please select at least {} option{}",
                    self.min_selections,
                    if self.min_selections == 1 { "" } else { "s" }
                ),
            });
        }
        if let Some(max) = self.max_selections {
            if indices.len() > max {
                return Err(InvalidResponse {
                    message: format!(
                        "Please select at most {} option{}",
                        max,
                        if max == 1 { "" } else { "s" }
                    ),
                });
            }
        }
        Ok(indices.to_vec())
    }

    /// Show the prompt and return selected indices (0-based).
    ///
    /// Returns an error if choices is empty.
    pub fn ask(&self, console: &mut Console) -> Result<Vec<usize>, InvalidResponse> {
        if self.choices.is_empty() {
            return Err(InvalidResponse {
                message: "No choices provided".to_string(),
            });
        }
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.ask_with_input(console, &mut handle)
    }

    /// Testable version of `ask()` that reads from a provided input source.
    pub fn ask_with_input<R: BufRead>(
        &self,
        console: &mut Console,
        input: &mut R,
    ) -> Result<Vec<usize>, InvalidResponse> {
        if self.choices.is_empty() {
            return Err(InvalidResponse {
                message: "No choices provided".to_string(),
            });
        }

        // Print the choice list
        let choices_display = self.format_choices();
        console.print_text(&choices_display);

        loop {
            let prompt_line = self.format_input_prompt();
            print!("{}", prompt_line);
            let _ = io::stdout().flush();

            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) => {
                    // EOF
                    if !self.defaults.is_empty() {
                        match self.validate_count(&self.defaults) {
                            Ok(indices) => return Ok(indices),
                            Err(_) => {
                                return Err(InvalidResponse {
                                    message: "No input provided".to_string(),
                                });
                            }
                        }
                    }
                    // Try empty selection (may succeed if min_selections == 0)
                    match self.validate_count(&[]) {
                        Ok(indices) => return Ok(indices),
                        Err(_) => {
                            return Err(InvalidResponse {
                                message: "No input provided".to_string(),
                            });
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(InvalidResponse {
                        message: format!("Input error: {}", e),
                    });
                }
            }

            match self.parse_input(&line) {
                Ok(indices) => return Ok(indices),
                Err(msg) => {
                    eprintln!("{}", msg);
                    continue;
                }
            }
        }
    }

    /// Show the prompt and return selected values.
    pub fn ask_values(&self, console: &mut Console) -> Result<Vec<String>, InvalidResponse> {
        let indices = self.ask(console)?;
        Ok(indices.iter().map(|&i| self.choices[i].clone()).collect())
    }

    /// Testable version of `ask_values()` that reads from a provided input source.
    pub fn ask_values_with_input<R: BufRead>(
        &self,
        console: &mut Console,
        input: &mut R,
    ) -> Result<Vec<String>, InvalidResponse> {
        let indices = self.ask_with_input(console, input)?;
        Ok(indices.iter().map(|&i| self.choices[i].clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // -- Simple prompt returns input ----------------------------------------

    #[test]
    fn test_simple_prompt_returns_input() {
        let p = Prompt::new("Enter name");
        let mut input = Cursor::new(b"Alice\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "Alice");
    }

    // -- Prompt with default (empty input returns default) ------------------

    #[test]
    fn test_prompt_with_default_empty_returns_default() {
        let p = Prompt::new("Enter name").with_default("Bob");
        let mut input = Cursor::new(b"\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "Bob");
    }

    #[test]
    fn test_prompt_with_default_non_empty_returns_input() {
        let p = Prompt::new("Enter name").with_default("Bob");
        let mut input = Cursor::new(b"Charlie\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "Charlie");
    }

    // -- Prompt with choices (valid choice accepted) ------------------------

    #[test]
    fn test_prompt_with_choices_valid() {
        let p = Prompt::new("Pick fruit").with_choices(vec![
            "apple".into(),
            "orange".into(),
            "pear".into(),
        ]);
        let mut input = Cursor::new(b"apple\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "apple");
    }

    // -- Prompt with choices (invalid choice rejected, loops) ---------------

    #[test]
    fn test_prompt_with_choices_invalid_then_valid() {
        let p = Prompt::new("Pick fruit").with_choices(vec![
            "apple".into(),
            "orange".into(),
            "pear".into(),
        ]);
        // First "banana" is invalid, then "orange" is valid
        let mut input = Cursor::new(b"banana\norange\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "orange");
    }

    // -- Case insensitive choices -------------------------------------------

    #[test]
    fn test_case_insensitive_choices() {
        let p = Prompt::new("Pick")
            .with_choices(vec!["Apple".into(), "Orange".into()])
            .with_case_sensitive(false);
        let mut input = Cursor::new(b"apple\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        // Should return the original-cased version
        assert_eq!(result, "Apple");
    }

    #[test]
    fn test_case_sensitive_choices_reject_wrong_case() {
        let p = Prompt::new("Pick")
            .with_choices(vec!["Apple".into(), "Orange".into()])
            .with_case_sensitive(true);
        // "apple" is wrong case; then "Apple" is correct
        let mut input = Cursor::new(b"apple\nApple\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "Apple");
    }

    // -- confirm() returns true/false ---------------------------------------

    #[test]
    fn test_confirm_yes() {
        let mut input = Cursor::new(b"y\n" as &[u8]);
        let result = confirm_with_input("Continue?", &mut input);
        assert!(result);
    }

    #[test]
    fn test_confirm_yes_full() {
        let mut input = Cursor::new(b"yes\n" as &[u8]);
        let result = confirm_with_input("Continue?", &mut input);
        assert!(result);
    }

    #[test]
    fn test_confirm_no() {
        let mut input = Cursor::new(b"n\n" as &[u8]);
        let result = confirm_with_input("Continue?", &mut input);
        assert!(!result);
    }

    #[test]
    fn test_confirm_no_full() {
        let mut input = Cursor::new(b"no\n" as &[u8]);
        let result = confirm_with_input("Continue?", &mut input);
        assert!(!result);
    }

    #[test]
    fn test_confirm_case_insensitive() {
        let mut input = Cursor::new(b"Y\n" as &[u8]);
        let result = confirm_with_input("Continue?", &mut input);
        assert!(result);
    }

    #[test]
    fn test_confirm_invalid_then_valid() {
        let mut input = Cursor::new(b"maybe\ny\n" as &[u8]);
        let result = confirm_with_input("Continue?", &mut input);
        assert!(result);
    }

    // -- ask_int() valid integer --------------------------------------------

    #[test]
    fn test_ask_int_valid() {
        let mut input = Cursor::new(b"42\n" as &[u8]);
        let result = ask_int_with_input("Enter number", &mut input);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_ask_int_negative() {
        let mut input = Cursor::new(b"-7\n" as &[u8]);
        let result = ask_int_with_input("Enter number", &mut input);
        assert_eq!(result, -7);
    }

    // -- ask_int() invalid input loops --------------------------------------

    #[test]
    fn test_ask_int_invalid_then_valid() {
        let mut input = Cursor::new(b"abc\n42\n" as &[u8]);
        let result = ask_int_with_input("Enter number", &mut input);
        assert_eq!(result, 42);
    }

    // -- ask_float() valid float --------------------------------------------

    #[test]
    fn test_ask_float_valid() {
        let mut input = Cursor::new(b"3.14\n" as &[u8]);
        let result = ask_float_with_input("Enter number", &mut input);
        assert!((result - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ask_float_integer_input() {
        let mut input = Cursor::new(b"7\n" as &[u8]);
        let result = ask_float_with_input("Enter number", &mut input);
        assert!((result - 7.0).abs() < f64::EPSILON);
    }

    // -- ask_float() invalid input loops ------------------------------------

    #[test]
    fn test_ask_float_invalid_then_valid() {
        let mut input = Cursor::new(b"xyz\n2.718\n" as &[u8]);
        let result = ask_float_with_input("Enter number", &mut input);
        assert!((result - 2.718).abs() < f64::EPSILON);
    }

    // -- Prompt text includes choices when show_choices is true --------------

    #[test]
    fn test_prompt_text_includes_choices() {
        let p = Prompt::new("Pick fruit")
            .with_choices(vec!["apple".into(), "orange".into()])
            .with_show_choices(true);
        let text = p.make_prompt();
        let plain = text.plain().to_string();
        assert!(plain.contains("[apple/orange]"));
    }

    #[test]
    fn test_prompt_text_hides_choices_when_disabled() {
        let p = Prompt::new("Pick fruit")
            .with_choices(vec!["apple".into(), "orange".into()])
            .with_show_choices(false);
        let text = p.make_prompt();
        let plain = text.plain().to_string();
        assert!(!plain.contains("apple"));
        assert!(!plain.contains("orange"));
    }

    // -- Prompt text includes default when show_default is true --------------

    #[test]
    fn test_prompt_text_includes_default() {
        let p = Prompt::new("Enter name")
            .with_default("World")
            .with_show_default(true);
        let text = p.make_prompt();
        let plain = text.plain().to_string();
        assert!(plain.contains("(World)"));
    }

    #[test]
    fn test_prompt_text_hides_default_when_disabled() {
        let p = Prompt::new("Enter name")
            .with_default("World")
            .with_show_default(false);
        let text = p.make_prompt();
        let plain = text.plain().to_string();
        assert!(!plain.contains("(World)"));
    }

    // -- Password flag (verify it's stored) ---------------------------------

    #[test]
    fn test_password_flag() {
        let p = Prompt::new("Password").with_password(true);
        assert!(p.password);

        let p2 = Prompt::new("Password").with_password(false);
        assert!(!p2.password);
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_builder_with_choices() {
        let p = Prompt::new("test").with_choices(vec!["a".into(), "b".into()]);
        assert_eq!(p.choices, Some(vec!["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn test_builder_with_default() {
        let p = Prompt::new("test").with_default("val");
        assert_eq!(p.default, Some("val".to_string()));
    }

    #[test]
    fn test_builder_with_case_sensitive() {
        let p = Prompt::new("test").with_case_sensitive(false);
        assert!(!p.case_sensitive);
    }

    #[test]
    fn test_builder_with_show_default() {
        let p = Prompt::new("test").with_show_default(false);
        assert!(!p.show_default);
    }

    #[test]
    fn test_builder_with_show_choices() {
        let p = Prompt::new("test").with_show_choices(false);
        assert!(!p.show_choices);
    }

    #[test]
    fn test_builder_with_password() {
        let p = Prompt::new("test").with_password(true);
        assert!(p.password);
    }

    #[test]
    fn test_builder_with_completions() {
        let p = Prompt::new("test").with_completions(vec!["foo".into(), "bar".into()]);
        assert_eq!(
            p.completions,
            Some(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn test_builder_completions_default_none() {
        let p = Prompt::new("test");
        assert!(p.completions.is_none());
    }

    // -- Prompt suffix is ": " ----------------------------------------------

    #[test]
    fn test_prompt_suffix() {
        let p = Prompt::new("Enter value");
        let text = p.make_prompt();
        let plain = text.plain().to_string();
        assert!(plain.ends_with(": "));
    }

    // -- Default on EOF -----------------------------------------------------

    #[test]
    fn test_default_on_eof() {
        let p = Prompt::new("Enter").with_default("fallback");
        let mut input = Cursor::new(b"" as &[u8]); // EOF immediately
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "fallback");
    }

    // -- No default, no choices, empty input --------------------------------

    #[test]
    fn test_no_default_empty_returns_empty() {
        let p = Prompt::new("Enter");
        let mut input = Cursor::new(b"\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "");
    }

    // -- Prompt with both choices and default --------------------------------

    #[test]
    fn test_prompt_text_choices_and_default() {
        let p = Prompt::new("Pick")
            .with_choices(vec!["a".into(), "b".into()])
            .with_default("a")
            .with_show_choices(true)
            .with_show_default(true);
        let text = p.make_prompt();
        let plain = text.plain().to_string();
        assert!(plain.contains("[a/b]"));
        assert!(plain.contains("(a)"));
        assert!(plain.ends_with(": "));
    }

    // -- Choices with default on empty input ---------------------------------

    #[test]
    fn test_choices_default_on_empty() {
        let p = Prompt::new("Pick")
            .with_choices(vec!["a".into(), "b".into()])
            .with_default("a");
        let mut input = Cursor::new(b"\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "a");
    }

    // -- Styled spans in prompt text ----------------------------------------

    #[test]
    fn test_prompt_has_styled_choices_span() {
        let p = Prompt::new("Pick")
            .with_choices(vec!["x".into(), "y".into()])
            .with_show_choices(true);
        let text = p.make_prompt();
        // Should have at least one span for the choices styling
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_prompt_has_styled_default_span() {
        let p = Prompt::new("Pick")
            .with_default("z")
            .with_show_default(true);
        let text = p.make_prompt();
        // Should have at least one span for the default styling
        assert!(!text.spans().is_empty());
    }

    // -- InvalidResponse ----------------------------------------------------

    #[test]
    fn test_invalid_response_display() {
        let err = InvalidResponse {
            message: "bad input".to_string(),
        };
        assert_eq!(format!("{}", err), "bad input");
    }

    #[test]
    fn test_invalid_response_debug() {
        let err = InvalidResponse {
            message: "bad".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("InvalidResponse"));
        assert!(debug.contains("bad"));
    }

    // -- Password mode (ask_with_input reads normally — password only affects ask()) --

    #[test]
    fn test_password_ask_with_input_reads_normally() {
        // ask_with_input always reads from the BufRead regardless of password flag.
        // This verifies the password flag doesn't break BufRead-based input.
        let p = Prompt::new("Password").with_password(true);
        let mut input = Cursor::new(b"secret123\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "secret123");
    }

    #[test]
    fn test_password_with_default_on_empty() {
        let p = Prompt::new("Password")
            .with_password(true)
            .with_default("default_pass");
        let mut input = Cursor::new(b"\n" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "default_pass");
    }

    #[test]
    fn test_password_with_default_on_eof() {
        let p = Prompt::new("Password")
            .with_password(true)
            .with_default("fallback");
        let mut input = Cursor::new(b"" as &[u8]);
        let result = p.ask_with_input(&mut input);
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_password_prompt_text_unchanged() {
        // Password mode should NOT affect the rendered prompt text
        let p1 = Prompt::new("Enter password").with_password(true);
        let p2 = Prompt::new("Enter password").with_password(false);
        let text1 = p1.make_prompt().plain().to_string();
        let text2 = p2.make_prompt().plain().to_string();
        assert_eq!(text1, text2);
    }

    // ===================================================================
    // Select tests
    // ===================================================================

    // -- Select: parse single number input ----------------------------------

    #[test]
    fn test_select_parse_single_number() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        assert_eq!(s.parse_input("2"), Ok(1)); // 1-based "2" -> 0-based 1
    }

    #[test]
    fn test_select_parse_first_choice() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]);
        assert_eq!(s.parse_input("1"), Ok(0));
    }

    #[test]
    fn test_select_parse_last_choice() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into(), "D".into()]);
        assert_eq!(s.parse_input("4"), Ok(3));
    }

    // -- Select: validate number in range -----------------------------------

    #[test]
    fn test_select_validate_in_range() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        assert!(s.parse_input("1").is_ok());
        assert!(s.parse_input("2").is_ok());
        assert!(s.parse_input("3").is_ok());
    }

    // -- Select: validate number out of range -------------------------------

    #[test]
    fn test_select_validate_out_of_range_high() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]);
        let result = s.parse_input("3");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("between 1 and 2"));
    }

    #[test]
    fn test_select_validate_out_of_range_zero() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]);
        let result = s.parse_input("0");
        assert!(result.is_err());
    }

    #[test]
    fn test_select_validate_not_a_number() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]);
        let result = s.parse_input("abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not a valid number"));
    }

    // -- Select: default selection when input is empty ----------------------

    #[test]
    fn test_select_default_on_empty() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_default(1);
        assert_eq!(s.parse_input(""), Ok(1));
    }

    #[test]
    fn test_select_no_default_on_empty() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]);
        let result = s.parse_input("");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("enter a number"));
    }

    // -- Select: handle whitespace in input ---------------------------------

    #[test]
    fn test_select_whitespace_input() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        assert_eq!(s.parse_input("  2  "), Ok(1));
    }

    // -- Select: empty choices list error -----------------------------------

    #[test]
    fn test_select_empty_choices() {
        let s = Select::new("Pick", vec![]);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"1\n" as &[u8]);
        let result = s.ask_with_input(&mut console, &mut input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "No choices provided");
    }

    // -- Select: display formatting -----------------------------------------

    #[test]
    fn test_select_format_choices() {
        let s = Select::new(
            "Select a color",
            vec!["Red".into(), "Green".into(), "Blue".into()],
        );
        let output = s.format_choices();
        assert!(output.contains("? Select a color:"));
        assert!(output.contains("  1) Red"));
        assert!(output.contains("  2) Green"));
        assert!(output.contains("  3) Blue"));
    }

    #[test]
    fn test_select_format_input_prompt_no_default() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        let prompt = s.format_input_prompt();
        assert_eq!(prompt, "Enter choice [1-3]: ");
    }

    #[test]
    fn test_select_format_input_prompt_with_default() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_default(1);
        let prompt = s.format_input_prompt();
        assert_eq!(prompt, "Enter choice [1-3] (2): ");
    }

    // -- Select: ask_with_input valid interaction ---------------------------

    #[test]
    fn test_select_ask_with_input_valid() {
        let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"2\n" as &[u8]);
        let result = s.ask_with_input(&mut console, &mut input);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_select_ask_with_input_invalid_then_valid() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"5\n1\n" as &[u8]);
        let result = s.ask_with_input(&mut console, &mut input);
        assert_eq!(result.unwrap(), 0);
    }

    // -- Select: ask_value --------------------------------------------------

    #[test]
    fn test_select_ask_value_with_input() {
        let s = Select::new("Pick", vec!["Red".into(), "Green".into(), "Blue".into()]);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"3\n" as &[u8]);
        let result = s.ask_value_with_input(&mut console, &mut input);
        assert_eq!(result.unwrap(), "Blue");
    }

    // -- Select: default on EOF ---------------------------------------------

    #[test]
    fn test_select_default_on_eof() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]).with_default(0);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"" as &[u8]);
        let result = s.ask_with_input(&mut console, &mut input);
        assert_eq!(result.unwrap(), 0);
    }

    // -- Select: builder methods --------------------------------------------

    #[test]
    fn test_select_builder_with_default() {
        let s = Select::new("Pick", vec!["A".into()]).with_default(0);
        assert_eq!(s.default, Some(0));
    }

    #[test]
    fn test_select_builder_with_style() {
        let style = Style::parse("red bold").unwrap();
        let s = Select::new("Pick", vec!["A".into()]).with_style(style);
        assert_eq!(s.style.bold(), Some(true));
    }

    #[test]
    fn test_select_builder_with_highlight_style() {
        let style = Style::parse("green").unwrap();
        let s = Select::new("Pick", vec!["A".into()]).with_highlight_style(style);
        assert!(s.highlight_style.color().is_some());
    }

    // ===================================================================
    // MultiSelect tests
    // ===================================================================

    // -- MultiSelect: parse comma-separated input ---------------------------

    #[test]
    fn test_multiselect_parse_comma_separated() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into(), "D".into()]);
        assert_eq!(ms.parse_input("1,3"), Ok(vec![0, 2]));
    }

    #[test]
    fn test_multiselect_parse_single() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
        assert_eq!(ms.parse_input("2"), Ok(vec![1]));
    }

    // -- MultiSelect: parse "all" keyword -----------------------------------

    #[test]
    fn test_multiselect_parse_all() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        assert_eq!(ms.parse_input("all"), Ok(vec![0, 1, 2]));
    }

    #[test]
    fn test_multiselect_parse_all_case_insensitive() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
        assert_eq!(ms.parse_input("ALL"), Ok(vec![0, 1]));
        assert_eq!(ms.parse_input("All"), Ok(vec![0, 1]));
    }

    // -- MultiSelect: handle whitespace in input ----------------------------

    #[test]
    fn test_multiselect_whitespace_input() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        assert_eq!(ms.parse_input("  1 , 3 "), Ok(vec![0, 2]));
    }

    #[test]
    fn test_multiselect_trailing_comma() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
        assert_eq!(ms.parse_input("1,2,"), Ok(vec![0, 1]));
    }

    // -- MultiSelect: default selection when input is empty -----------------

    #[test]
    fn test_multiselect_default_on_empty() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()])
            .with_defaults(vec![0, 2]);
        assert_eq!(ms.parse_input(""), Ok(vec![0, 2]));
    }

    // -- MultiSelect: min/max selection validation --------------------------

    #[test]
    fn test_multiselect_min_validation() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_min(2);
        let result = ms.parse_input("1");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("at least 2"));
    }

    #[test]
    fn test_multiselect_min_satisfied() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_min(2);
        assert_eq!(ms.parse_input("1,2"), Ok(vec![0, 1]));
    }

    #[test]
    fn test_multiselect_max_validation() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_max(1);
        let result = ms.parse_input("1,2");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("at most 1"));
    }

    #[test]
    fn test_multiselect_max_satisfied() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_max(2);
        assert_eq!(ms.parse_input("1,3"), Ok(vec![0, 2]));
    }

    #[test]
    fn test_multiselect_min_max_combined() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into(), "D".into()])
            .with_min(1)
            .with_max(3);
        assert!(ms.parse_input("").is_err()); // 0 < min
        assert!(ms.parse_input("1").is_ok()); // 1 >= min
        assert!(ms.parse_input("1,2,3").is_ok()); // 3 <= max
        assert!(ms.parse_input("1,2,3,4").is_err()); // 4 > max
    }

    // -- MultiSelect: empty choices list error ------------------------------

    #[test]
    fn test_multiselect_empty_choices() {
        let ms = MultiSelect::new("Pick", vec![]);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"1\n" as &[u8]);
        let result = ms.ask_with_input(&mut console, &mut input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "No choices provided");
    }

    // -- MultiSelect: validate out of range ---------------------------------

    #[test]
    fn test_multiselect_out_of_range() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
        let result = ms.parse_input("3");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("out of range"));
    }

    #[test]
    fn test_multiselect_zero() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
        let result = ms.parse_input("0");
        assert!(result.is_err());
    }

    // -- MultiSelect: display formatting ------------------------------------

    #[test]
    fn test_multiselect_format_choices() {
        let ms = MultiSelect::new(
            "Select colors",
            vec!["Red".into(), "Green".into(), "Blue".into()],
        );
        let output = ms.format_choices();
        assert!(output.contains("? Select colors (comma-separated):"));
        assert!(output.contains("  1) Red"));
        assert!(output.contains("  2) Green"));
        assert!(output.contains("  3) Blue"));
    }

    #[test]
    fn test_multiselect_format_input_prompt_no_defaults() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        let prompt = ms.format_input_prompt();
        assert_eq!(prompt, "Enter choices [1-3, e.g. 1,3]: ");
    }

    #[test]
    fn test_multiselect_format_input_prompt_with_defaults() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()])
            .with_defaults(vec![0, 2]);
        let prompt = ms.format_input_prompt();
        assert_eq!(prompt, "Enter choices [1-3, e.g. 1,3] (1,3): ");
    }

    // -- MultiSelect: ask_with_input valid interaction ----------------------

    #[test]
    fn test_multiselect_ask_with_input_valid() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"1,3\n" as &[u8]);
        let result = ms.ask_with_input(&mut console, &mut input);
        assert_eq!(result.unwrap(), vec![0, 2]);
    }

    #[test]
    fn test_multiselect_ask_with_input_invalid_then_valid() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]).with_min(1);
        let mut console = Console::builder().quiet(true).build();
        // First line is empty (fails min), then "2" succeeds
        let mut input = Cursor::new(b"\n2\n" as &[u8]);
        let result = ms.ask_with_input(&mut console, &mut input);
        assert_eq!(result.unwrap(), vec![1]);
    }

    // -- MultiSelect: ask_values --------------------------------------------

    #[test]
    fn test_multiselect_ask_values_with_input() {
        let ms = MultiSelect::new("Pick", vec!["Red".into(), "Green".into(), "Blue".into()]);
        let mut console = Console::builder().quiet(true).build();
        let mut input = Cursor::new(b"1,3\n" as &[u8]);
        let result = ms.ask_values_with_input(&mut console, &mut input);
        assert_eq!(result.unwrap(), vec!["Red".to_string(), "Blue".to_string()]);
    }

    // -- MultiSelect: deduplicate input -------------------------------------

    #[test]
    fn test_multiselect_deduplicate() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
        assert_eq!(ms.parse_input("1,1,2"), Ok(vec![0, 1]));
    }

    // -- MultiSelect: empty input with min=0 succeeds -----------------------

    #[test]
    fn test_multiselect_empty_input_min_zero() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
        assert_eq!(ms.parse_input(""), Ok(vec![]));
    }

    // -- MultiSelect: builder methods ---------------------------------------

    #[test]
    fn test_multiselect_builder_with_defaults() {
        let ms = MultiSelect::new("Pick", vec!["A".into()]).with_defaults(vec![0]);
        assert_eq!(ms.defaults, vec![0]);
    }

    #[test]
    fn test_multiselect_builder_with_min() {
        let ms = MultiSelect::new("Pick", vec!["A".into()]).with_min(1);
        assert_eq!(ms.min_selections, 1);
    }

    #[test]
    fn test_multiselect_builder_with_max() {
        let ms = MultiSelect::new("Pick", vec!["A".into()]).with_max(3);
        assert_eq!(ms.max_selections, Some(3));
    }

    #[test]
    fn test_multiselect_builder_with_style() {
        let style = Style::parse("red bold").unwrap();
        let ms = MultiSelect::new("Pick", vec!["A".into()]).with_style(style);
        assert_eq!(ms.style.bold(), Some(true));
    }

    #[test]
    fn test_multiselect_builder_with_highlight_style() {
        let style = Style::parse("green").unwrap();
        let ms = MultiSelect::new("Pick", vec!["A".into()]).with_highlight_style(style);
        assert!(ms.highlight_style.color().is_some());
    }

    // -- Select: display via console capture --------------------------------

    #[test]
    fn test_select_display_via_capture() {
        let s = Select::new(
            "Pick a fruit",
            vec!["Apple".into(), "Banana".into(), "Cherry".into()],
        );
        let mut console = Console::builder().width(80).force_terminal(true).build();
        console.begin_capture();
        console.print_text(&s.format_choices());
        let captured = console.end_capture();
        assert!(captured.contains("? Pick a fruit:"));
        assert!(captured.contains("1) Apple"));
        assert!(captured.contains("2) Banana"));
        assert!(captured.contains("3) Cherry"));
    }

    // -- MultiSelect: display via console capture ---------------------------

    #[test]
    fn test_multiselect_display_via_capture() {
        let ms = MultiSelect::new(
            "Pick colors",
            vec!["Red".into(), "Green".into(), "Blue".into(), "Yellow".into()],
        );
        let mut console = Console::builder().width(80).force_terminal(true).build();
        console.begin_capture();
        console.print_text(&ms.format_choices());
        let captured = console.end_capture();
        assert!(captured.contains("? Pick colors (comma-separated):"));
        assert!(captured.contains("1) Red"));
        assert!(captured.contains("2) Green"));
        assert!(captured.contains("3) Blue"));
        assert!(captured.contains("4) Yellow"));
    }

    // -- MultiSelect: "all" with max constraint -----------------------------

    #[test]
    fn test_multiselect_all_with_max() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_max(2);
        let result = ms.parse_input("all");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("at most 2"));
    }

    // -- Select: negative number input --------------------------------------

    #[test]
    fn test_select_negative_number() {
        let s = Select::new("Pick", vec!["A".into(), "B".into()]);
        let result = s.parse_input("-1");
        assert!(result.is_err());
    }

    // -- MultiSelect: not a number in list ----------------------------------

    #[test]
    fn test_multiselect_invalid_number_in_list() {
        let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
        let result = ms.parse_input("1,abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not a valid number"));
    }
}
