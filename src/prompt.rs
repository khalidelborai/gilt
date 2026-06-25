//! Interactive prompt module for styled user input with validation, choices, and defaults.
//!
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
// Type aliases used in Prompt to keep field types readable
// ---------------------------------------------------------------------------

/// Callback invoked when a validation error occurs.
pub type ValidateErrorHook = Box<dyn Fn(&str)>;

// ---------------------------------------------------------------------------
// Helper: print an error using the prompt.invalid theme style
// ---------------------------------------------------------------------------

/// Print an error message using the `prompt.invalid` theme style (red by default).
///
/// Falls back to `Style::parse("red")` — matching the theme default for
/// `prompt.invalid` — when the theme key is not present. Under normal
/// operation the key is always found, so this branch is unreachable.
fn print_invalid_error(console: &mut Console, msg: &str) {
    let style = console
        .get_style("prompt.invalid")
        .unwrap_or_else(|_| Style::parse("red"));
    let t = Text::new(msg, style);
    console.print(&t);
}

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
    /// The suffix appended at the end of the prompt (default: `": "`).
    pub prompt_suffix: String,
    /// Optional callback invoked before printing the prompt each iteration.
    pub(crate) pre_prompt: Option<Box<dyn Fn()>>,
    /// Optional callback invoked when a validation error occurs, in addition
    /// to the standard error console output.
    pub(crate) on_validate_error: Option<ValidateErrorHook>,
    /// The console used for rendering prompt text.
    console: Console,
}

impl Prompt {
    /// Create a new prompt with the given text.
    ///
    /// The prompt string is parsed as gilt markup.
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
            prompt_suffix: ": ".to_string(),
            pre_prompt: None,
            on_validate_error: None,
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

    /// Set the suffix appended at the end of the prompt text (default: `": "`).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prompt::Prompt;
    ///
    /// let p = Prompt::new("Enter value").with_suffix(" -> ");
    /// ```
    #[must_use]
    pub fn with_suffix(mut self, suffix: &str) -> Self {
        self.prompt_suffix = suffix.to_string();
        self
    }

    /// Set a callback invoked before printing the prompt on each iteration.
    ///
    /// This is called once per prompt display (including re-prompts after
    /// validation errors).
    #[must_use]
    pub fn with_pre_prompt<F: Fn() + 'static>(mut self, f: F) -> Self {
        self.pre_prompt = Some(Box::new(f));
        self
    }

    /// Set a callback invoked when a validation error occurs.
    ///
    /// The callback receives the error message text. It is called in addition
    /// to the standard error console output, not instead of it.
    #[must_use]
    pub fn with_on_validate_error<F: Fn(&str) + 'static>(mut self, f: F) -> Self {
        self.on_validate_error = Some(Box::new(f));
        self
    }

    /// Build the prompt `Text` including choices and default annotations.
    ///
    /// Format: `"prompt [choice1/choice2/...] (default): "`
    ///
    /// Choices are styled with the `prompt.choices` theme style (magenta bold).
    /// The default annotation is styled with the `prompt.default` theme style
    /// (cyan bold). Both map to the canonical Python rich theme names.
    pub fn make_prompt(&self) -> Text {
        // Hoist style construction here so callers that pre-build the prompt
        // Text before a retry loop only parse styles once.
        use std::sync::LazyLock;
        static CHOICES_STYLE: LazyLock<Style> = LazyLock::new(|| Style::parse("magenta bold"));
        static DEFAULT_STYLE: LazyLock<Style> = LazyLock::new(|| Style::parse("cyan bold"));

        let mut prompt = self.prompt_text.clone();
        prompt.end = String::new();

        if self.show_choices {
            if let Some(ref choices) = self.choices {
                let choices_str = format!("[{}]", choices.join("/"));
                // "prompt.choices" theme name — magenta bold
                prompt.append_str(" ", None);
                prompt.append_str(&choices_str, Some(CHOICES_STYLE.clone()));
            }
        }

        if self.show_default {
            if let Some(ref default) = self.default {
                let default_str = format!("({})", default);
                // "prompt.default" theme name — cyan bold
                prompt.append_str(" ", None);
                prompt.append_str(&default_str, Some(DEFAULT_STYLE.clone()));
            }
        }

        prompt.append_str(&self.prompt_suffix, None);

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
    ///
    /// Finding #1: the prompt is printed via the console's styled rendering
    /// pipeline so markup styling is preserved. `&mut self` is required because
    /// the console's `write_segments` method takes `&mut self`.
    pub fn ask_with_input<R: BufRead>(&mut self, input: &mut R) -> String {
        // Build the prompt Text once before the retry loop (finding #7).
        let prompt = self.make_prompt();

        // Render the styled prompt to ANSI bytes without a trailing newline
        // (finding #1). We capture the console output then write it raw.
        let ansi_prompt: String = {
            self.console.begin_capture();
            // Temporarily push the prompt as a renderable segment sequence.
            // Use write_segments directly via begin_capture + end_capture trick:
            // print via console but strip the auto-appended newline.
            self.console.print(&prompt);
            let captured = self.console.end_capture();
            // Strip trailing newlines (Console::print appends \n; on CRLF
            // platforms a \r may also be present — strip both, rich parity).
            captured.trim_end_matches(['\n', '\r']).to_string()
        };

        loop {
            // Item 6: call pre_prompt hook before each iteration.
            if let Some(ref hook) = self.pre_prompt {
                hook();
            }

            print!("{}", ansi_prompt);
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

            // Fix #6: correct CRLF trim order — strip both \r and \n in any order.
            let trimmed = line.trim_end_matches(['\n', '\r']);
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
                    // Item 2: use prompt.invalid theme style instead of hardcoded markup.
                    let err_msg = "Please select one of the available options";
                    print_invalid_error(&mut self.console, err_msg);
                    // Item 6: call on_validate_error hook.
                    if let Some(ref hook) = self.on_validate_error {
                        hook(err_msg);
                    }
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
    pub fn ask(&mut self) -> String {
        #[cfg(feature = "interactive")]
        if self.password {
            return self.ask_password();
        }
        #[cfg(not(feature = "interactive"))]
        if self.password {
            // Fall back to regular input when rpassword is unavailable.
            // WARNING: input will be visible on screen.
            self.console.print_text(
                "[bold yellow]warning:[/] gilt built without `interactive` feature; password input will be visible",
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

    /// Attach a typed converter function to this `Prompt`, producing a
    /// [`TypedPrompt<T, F>`] that loops until the converter succeeds.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prompt::Prompt;
    /// use std::io::Cursor;
    ///
    /// let mut tp = Prompt::new("Enter a u16")
    ///     .with_converter(|s: &str| s.parse::<u16>().map_err(|e| e.to_string()));
    /// let mut input = Cursor::new(b"abc\n42\n" as &[u8]);
    /// let value = tp.ask_with_input(&mut input).unwrap();
    /// assert_eq!(value, 42u16);
    /// ```
    pub fn with_converter<T, F>(self, converter: F) -> TypedPrompt<T, F>
    where
        F: Fn(&str) -> Result<T, String>,
    {
        TypedPrompt {
            prompt: self,
            converter,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Readline-based input loop with tab-completion.
    #[cfg(feature = "readline")]
    fn ask_readline(&mut self) -> String {
        let candidates = self.completions.clone().unwrap_or_default();
        let helper = ListCompleter { candidates };
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut editor = rustyline::Editor::with_config(config).expect("Failed to create editor");
        editor.set_helper(Some(helper));

        loop {
            // Item 6: call pre_prompt hook before each iteration.
            if let Some(ref hook) = self.pre_prompt {
                hook();
            }

            let prompt = self.make_prompt();
            let prompt_str = prompt.plain().to_string();

            match editor.readline(&prompt_str) {
                Ok(line) => {
                    // Fix #6: correct CRLF trim.
                    let value = line.trim_end_matches(['\n', '\r']).to_string();

                    // Empty input: return default if available
                    if value.trim().is_empty() {
                        if let Some(ref default) = self.default {
                            return default.clone();
                        }
                    }

                    // Validate against choices
                    if self.choices.is_some() {
                        if !self.check_choice(&value) {
                            // Item 2: use theme style for error message.
                            let err_msg = "Please select one of the available options";
                            print_invalid_error(&mut self.console, err_msg);
                            // Item 6: call on_validate_error hook.
                            if let Some(ref hook) = self.on_validate_error {
                                hook(err_msg);
                            }
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
    fn ask_password(&mut self) -> String {
        // Item 3: use styled pipeline instead of plain() for the prompt text.
        let prompt_text = self.make_prompt();
        let ansi_prompt: String = {
            self.console.begin_capture();
            self.console.print(&prompt_text);
            let captured = self.console.end_capture();
            captured.trim_end_matches(['\n', '\r']).to_string()
        };

        loop {
            // Item 6: call pre_prompt hook before each iteration.
            if let Some(ref hook) = self.pre_prompt {
                hook();
            }

            print!("{}", ansi_prompt);
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
                    // Item 2: use theme style for error message.
                    let err_msg = "Please select one of the available options";
                    print_invalid_error(&mut self.console, err_msg);
                    // Item 6: call on_validate_error hook.
                    if let Some(ref hook) = self.on_validate_error {
                        hook(err_msg);
                    }
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
    confirm_with_default(prompt, None)
}

/// Ask a yes/no confirmation question with an optional default.
///
/// When `default` is `Some(true)`, the choices are shown as `[Y/n]` and blank
/// input returns `true`. When `Some(false)`, choices are `[y/N]` and blank
/// input returns `false`. When `None`, choices are `[y/n]` and blank/EOF
/// returns `false`.
pub fn confirm_with_default(prompt: &str, default: Option<bool>) -> bool {
    confirm_with_input_and_default(prompt, default, &mut io::stdin().lock())
}

/// Testable version of `confirm()` that reads from a provided input source.
///
/// Accepts `"y"`, `"yes"`, `"n"`, `"no"` (case-insensitive).
/// When `default` is set, blank/EOF input returns the default; otherwise
/// blank input prompts again.
pub fn confirm_with_input<R: BufRead>(prompt: &str, input: &mut R) -> bool {
    confirm_with_input_and_default(prompt, None, input)
}

/// Testable version of `confirm_with_default()` that reads from a provided input source.
pub fn confirm_with_input_and_default<R: BufRead>(
    prompt: &str,
    default: Option<bool>,
    input: &mut R,
) -> bool {
    // Item 8: render choices through the styled pipeline so the default
    // letter is bold.  Build the ANSI prompt string once before the loop
    // using a capture console so the bold escape is embedded in the output.
    let mut render_console = Console::new();
    let ansi_full_prompt: String = {
        render_console.begin_capture();
        // Render the static question text first.
        let question_text = crate::markup::render(prompt, crate::style::Style::null())
            .unwrap_or_else(|_| Text::new(prompt, crate::style::Style::null()));
        render_console.print(&question_text);
        // Strip the trailing newline so we can concatenate the choices on
        // the same line.
        let question_captured = render_console.end_capture();
        let question_part = question_captured.trim_end_matches(['\n', '\r']).to_string();

        // Build the choices Text with the default letter bold-styled.
        let choices_markup = match default {
            Some(true) => "[bold]Y[/bold]/n",
            Some(false) => "y/[bold]N[/bold]",
            None => "y/n",
        };
        render_console.begin_capture();
        let choices_text = crate::markup::render(choices_markup, crate::style::Style::null())
            .unwrap_or_else(|_| Text::new(choices_markup, crate::style::Style::null()));
        render_console.print(&choices_text);
        let choices_captured = render_console.end_capture();
        let choices_part = choices_captured.trim_end_matches(['\n', '\r']).to_string();

        format!("{} [{}]: ", question_part, choices_part)
    };

    // Error messages are routed through a stderr console so they pick up
    // gilt styling when stderr is a terminal.
    let mut err_console = Console::stderr();

    loop {
        print!("{}", ansi_full_prompt);
        let _ = io::stdout().flush();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => {
                // EOF: return default if set, otherwise false
                return default.unwrap_or(false);
            }
            Ok(_) => {}
            Err(_) => return default.unwrap_or(false),
        }

        let value = line.trim().to_lowercase();
        match value.as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            "" => {
                // Blank input: return default when set, else re-prompt
                if let Some(d) = default {
                    return d;
                }
                // No default — show error and loop.
                // Item 2: use theme style for error message.
                print_invalid_error(&mut err_console, "Please enter Y or N");
                continue;
            }
            _ => {
                // Item 2: use theme style for error message.
                print_invalid_error(&mut err_console, "Please enter Y or N");
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
    let mut err_console = Console::stderr();
    // Item 5: use console render pipeline to get ANSI output.
    let ansi_prompt: String = {
        let mut render_console = Console::new();
        let prompt_text = Prompt::new(prompt).make_prompt();
        render_console.begin_capture();
        render_console.print(&prompt_text);
        let captured = render_console.end_capture();
        captured.trim_end_matches(['\n', '\r']).to_string()
    };
    loop {
        print!("{}", ansi_prompt);
        let _ = io::stdout().flush();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => {
                // EOF: terminate and return 0 (consistent with confirm/ask EOF
                // semantics — no default available for numeric prompts).
                return 0;
            }
            Ok(_) => {}
            Err(_) => {
                // Item 2: use theme style for error message.
                print_invalid_error(&mut err_console, "Please enter a valid integer number");
                continue;
            }
        }

        match line.trim().parse::<i64>() {
            Ok(v) => return v,
            Err(_) => {
                // Item 2: use theme style for error message.
                print_invalid_error(&mut err_console, "Please enter a valid integer number");
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
    let mut err_console = Console::stderr();
    // Item 5: use console render pipeline to get ANSI output.
    let ansi_prompt: String = {
        let mut render_console = Console::new();
        let prompt_text = Prompt::new(prompt).make_prompt();
        render_console.begin_capture();
        render_console.print(&prompt_text);
        let captured = render_console.end_capture();
        captured.trim_end_matches(['\n', '\r']).to_string()
    };
    loop {
        print!("{}", ansi_prompt);
        let _ = io::stdout().flush();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => {
                // EOF: terminate and return 0.0 (mirrors ask_int EOF semantics).
                return 0.0;
            }
            Ok(_) => {}
            Err(_) => {
                // Item 2: use theme style for error message.
                print_invalid_error(&mut err_console, "Please enter a valid number");
                continue;
            }
        }

        match line.trim().parse::<f64>() {
            Ok(v) => return v,
            Err(_) => {
                // Item 2: use theme style for error message.
                print_invalid_error(&mut err_console, "Please enter a valid number");
                continue;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// TypedPrompt
// ---------------------------------------------------------------------------

/// A [`Prompt`] paired with a converter function that maps `&str -> Result<T, String>`.
///
/// Created via [`Prompt::with_converter`]. The prompt loops until the converter
/// succeeds, showing the error message on each bad attempt.
///
/// # Examples
///
/// ```
/// use gilt::prompt::Prompt;
/// use std::io::Cursor;
///
/// // Parse a u16, re-prompting on invalid input
/// let mut tp = Prompt::new("Port number")
///     .with_converter(|s: &str| s.parse::<u16>().map_err(|e| e.to_string()));
///
/// let mut input = Cursor::new(b"not_a_number\n8080\n" as &[u8]);
/// let port = tp.ask_with_input(&mut input).unwrap();
/// assert_eq!(port, 8080u16);
/// ```
pub struct TypedPrompt<T, F>
where
    F: Fn(&str) -> Result<T, String>,
{
    prompt: Prompt,
    converter: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> TypedPrompt<T, F>
where
    F: Fn(&str) -> Result<T, String>,
{
    /// Read a line from `input`, convert it, loop on errors.
    ///
    /// Returns `Ok(T)` on success. Returns `Err` only on unexpected I/O errors
    /// at EOF when there is no default (consistent with the underlying
    /// `Prompt::ask_with_input` EOF handling).
    pub fn ask_with_input<R: BufRead>(&mut self, input: &mut R) -> io::Result<T> {
        let prompt_text = self.prompt.make_prompt();
        let ansi_prompt: String = {
            self.prompt.console.begin_capture();
            self.prompt.console.print(&prompt_text);
            let captured = self.prompt.console.end_capture();
            captured.trim_end_matches(['\n', '\r']).to_string()
        };

        let mut err_console = Console::stderr();

        loop {
            print!("{}", ansi_prompt);
            let _ = io::stdout().flush();

            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) => {
                    // EOF — if there is a default, try converting it
                    if let Some(ref default) = self.prompt.default {
                        match (self.converter)(default) {
                            Ok(v) => return Ok(v),
                            Err(msg) => {
                                // Item 2: use theme style for error message.
                                print_invalid_error(&mut err_console, &msg);
                                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, msg));
                            }
                        }
                    }
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "unexpected EOF",
                    ));
                }
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            let trimmed = line.trim_end_matches(['\n', '\r']);

            // Empty input: use default if available
            let value = if trimmed.trim().is_empty() {
                if let Some(ref default) = self.prompt.default {
                    default.as_str()
                } else {
                    trimmed
                }
            } else {
                trimmed
            };

            match (self.converter)(value) {
                Ok(v) => return Ok(v),
                Err(msg) => {
                    // Item 2: use theme style for error message.
                    print_invalid_error(&mut err_console, &msg);
                    // loop again
                }
            }
        }
    }

    /// Read from stdin. Convenience wrapper around
    /// [`ask_with_input`](Self::ask_with_input).
    pub fn ask(&mut self) -> io::Result<T> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.ask_with_input(&mut handle)
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
/// **Note (finding #8):** `Select` is a gilt extension with **no direct
/// counterpart in Python `rich`**. Rich's `Prompt` accepts free-form choices;
/// the numbered-list selection UI is a gilt addition.
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
            style: Style::parse("bold"),
            highlight_style: Style::parse("cyan bold"),
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
                    // Item 2: use theme style for error message.
                    print_invalid_error(&mut Console::stderr(), &msg.message);
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
/// **Note (finding #8):** `MultiSelect` is a gilt extension with **no direct
/// counterpart in Python `rich`**. Rich's multi-selection UI is provided by
/// external libraries; the numbered multi-select is a gilt addition.
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
            style: Style::parse("bold"),
            highlight_style: Style::parse("cyan bold"),
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
                    // Item 2: use theme style for error message.
                    print_invalid_error(&mut Console::stderr(), &msg.message);
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

// ---------------------------------------------------------------------------
// IntPrompt — typed integer prompt with optional default
// ---------------------------------------------------------------------------

/// A prompt that reads an integer (`i64`), re-prompting on invalid input.
///
/// On empty input or EOF, returns the `default` (if set) or `0`.
///
/// # Examples
///
/// ```
/// use gilt::prompt::IntPrompt;
/// use std::io::Cursor;
///
/// let mut p = IntPrompt::new("Enter age").with_default(18);
/// let mut input = Cursor::new(b"\n" as &[u8]);
/// assert_eq!(p.ask_with_input(&mut input), 18);
/// ```
#[non_exhaustive]
pub struct IntPrompt {
    /// The underlying `Prompt` used for rendering.
    pub prompt: Prompt,
    /// Optional default value returned on empty/EOF input.
    pub default: Option<i64>,
}

impl IntPrompt {
    /// Create a new `IntPrompt` with the given prompt text.
    pub fn new(prompt_text: &str) -> Self {
        IntPrompt {
            prompt: Prompt::new(prompt_text),
            default: None,
        }
    }

    /// Set the default value returned on empty or EOF input.
    #[must_use]
    pub fn with_default(mut self, default: i64) -> Self {
        self.default = Some(default);
        self
    }

    /// Read an integer from the provided reader, re-prompting on invalid input.
    ///
    /// Returns the default on empty input or EOF; returns `0` when there is
    /// no default and EOF is encountered.
    pub fn ask_with_input<R: BufRead>(&mut self, input: &mut R) -> i64 {
        let mut err_console = Console::stderr();
        // Render the prompt once using the styled pipeline.
        let ansi_prompt: String = {
            let prompt_text = self.prompt.make_prompt();
            self.prompt.console.begin_capture();
            self.prompt.console.print(&prompt_text);
            let captured = self.prompt.console.end_capture();
            captured.trim_end_matches(['\n', '\r']).to_string()
        };

        loop {
            print!("{}", ansi_prompt);
            let _ = io::stdout().flush();

            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) => {
                    return self.default.unwrap_or(0);
                }
                Ok(_) => {}
                Err(_) => {
                    print_invalid_error(&mut err_console, "Please enter a valid integer number");
                    continue;
                }
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                if let Some(d) = self.default {
                    return d;
                }
            }

            match trimmed.parse::<i64>() {
                Ok(v) => return v,
                Err(_) => {
                    print_invalid_error(&mut err_console, "Please enter a valid integer number");
                }
            }
        }
    }

    /// Read an integer from stdin.
    pub fn ask(&mut self) -> i64 {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.ask_with_input(&mut handle)
    }
}

// ---------------------------------------------------------------------------
// FloatPrompt — typed float prompt with optional default
// ---------------------------------------------------------------------------

/// A prompt that reads a float (`f64`), re-prompting on invalid input.
///
/// On empty input or EOF, returns the `default` (if set) or `0.0`.
///
/// # Examples
///
/// ```
/// use gilt::prompt::FloatPrompt;
/// use std::io::Cursor;
///
/// let mut p = FloatPrompt::new("Enter rate").with_default(1.5);
/// let mut input = Cursor::new(b"\n" as &[u8]);
/// assert!((p.ask_with_input(&mut input) - 1.5).abs() < f64::EPSILON);
/// ```
#[non_exhaustive]
pub struct FloatPrompt {
    /// The underlying `Prompt` used for rendering.
    pub prompt: Prompt,
    /// Optional default value returned on empty/EOF input.
    pub default: Option<f64>,
}

impl FloatPrompt {
    /// Create a new `FloatPrompt` with the given prompt text.
    pub fn new(prompt_text: &str) -> Self {
        FloatPrompt {
            prompt: Prompt::new(prompt_text),
            default: None,
        }
    }

    /// Set the default value returned on empty or EOF input.
    #[must_use]
    pub fn with_default(mut self, default: f64) -> Self {
        self.default = Some(default);
        self
    }

    /// Read a float from the provided reader, re-prompting on invalid input.
    ///
    /// Returns the default on empty input or EOF; returns `0.0` when there is
    /// no default and EOF is encountered.
    pub fn ask_with_input<R: BufRead>(&mut self, input: &mut R) -> f64 {
        let mut err_console = Console::stderr();
        // Render the prompt once using the styled pipeline.
        let ansi_prompt: String = {
            let prompt_text = self.prompt.make_prompt();
            self.prompt.console.begin_capture();
            self.prompt.console.print(&prompt_text);
            let captured = self.prompt.console.end_capture();
            captured.trim_end_matches(['\n', '\r']).to_string()
        };

        loop {
            print!("{}", ansi_prompt);
            let _ = io::stdout().flush();

            let mut line = String::new();
            match input.read_line(&mut line) {
                Ok(0) => {
                    return self.default.unwrap_or(0.0);
                }
                Ok(_) => {}
                Err(_) => {
                    print_invalid_error(&mut err_console, "Please enter a valid number");
                    continue;
                }
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                if let Some(d) = self.default {
                    return d;
                }
            }

            match trimmed.parse::<f64>() {
                Ok(v) => return v,
                Err(_) => {
                    print_invalid_error(&mut err_console, "Please enter a valid number");
                }
            }
        }
    }

    /// Read a float from stdin.
    pub fn ask(&mut self) -> f64 {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.ask_with_input(&mut handle)
    }
}

// ---------------------------------------------------------------------------
// Confirm — yes/no prompt struct with optional default
// ---------------------------------------------------------------------------

/// A yes/no confirmation prompt struct.
///
/// Separate from the [`confirm_with_input_and_default`] free function; this
/// struct provides a builder API and a testable `ask_with_input` method.
///
/// # Examples
///
/// ```
/// use gilt::prompt::Confirm;
/// use std::io::Cursor;
///
/// let mut c = Confirm::new("Continue?").with_default(true);
/// let mut input = Cursor::new(b"\n" as &[u8]);
/// assert!(c.ask_with_input(&mut input));
/// ```
#[non_exhaustive]
pub struct Confirm {
    /// The prompt question text.
    pub prompt: String,
    /// Optional default: `Some(true)` → `[Y/n]`, `Some(false)` → `[y/N]`, `None` → `[y/n]`.
    pub default: Option<bool>,
}

impl Confirm {
    /// Create a new `Confirm` prompt with the given question text.
    pub fn new(prompt: &str) -> Self {
        Confirm {
            prompt: prompt.to_string(),
            default: None,
        }
    }

    /// Set the default value.
    #[must_use]
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Read a yes/no answer from the provided reader.
    ///
    /// Returns the `default` on empty/EOF input. When there is no default,
    /// blank input re-prompts and EOF returns `false`.
    pub fn ask_with_input<R: BufRead>(&mut self, input: &mut R) -> bool {
        confirm_with_input_and_default(&self.prompt, self.default, input)
    }

    /// Read a yes/no answer from stdin.
    pub fn ask(&mut self) -> bool {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.ask_with_input(&mut handle)
    }
}

#[cfg(test)]
#[path = "prompt_tests.rs"]
mod tests;
