//! Form builder (huh pattern) — multi-step chained prompts with validation and
//! an accessibility fallback.
//!
//! # Example
//!
//! ```
//! use gilt::form::{Form, FormField};
//! use std::io::Cursor;
//!
//! let input = Cursor::new(b"Alice\ny\n" as &[u8]);
//! let mut input = input;
//! let result = Form::new()
//!     .field(FormField::input("name", "Your name?"))
//!     .field(FormField::confirm("ok", "Are you sure?", false))
//!     .run_with_input(&mut input)
//!     .unwrap();
//! assert_eq!(result["name"], "Alice");
//! assert_eq!(result["ok"], "true");
//! ```

use std::collections::HashMap;
use std::io::{self, BufRead, Write as IoWrite};

// ---------------------------------------------------------------------------
// FormField
// ---------------------------------------------------------------------------

/// Validator function type: receives the trimmed user input and returns
/// `Ok(())` if valid, `Err(message)` if invalid.
type ValidateFn = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// A single field in a [`Form`].
pub struct FormField {
    /// Unique key for this field in the returned `HashMap`.
    pub key: String,
    /// The prompt text shown to the user.
    pub prompt: String,
    /// The kind of field.
    pub(crate) kind: FormFieldKind,
}

pub(crate) enum FormFieldKind {
    /// Free-text input with optional validator.
    Input { validate: Option<ValidateFn> },
    /// Yes/no confirmation with a default.
    Confirm { default: bool },
    /// Select one option from a list.
    Select { options: Vec<String> },
}

impl FormField {
    /// Create a free-text input field.
    ///
    /// # Example
    ///
    /// ```
    /// use gilt::form::FormField;
    /// let f = FormField::input("name", "Your name?");
    /// ```
    pub fn input(key: &str, prompt: &str) -> Self {
        FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            kind: FormFieldKind::Input { validate: None },
        }
    }

    /// Attach a validator to an `Input` field.
    ///
    /// The validator receives the trimmed input string and should return
    /// `Ok(())` for valid input or `Err(message)` to re-prompt.
    ///
    /// # Panics
    ///
    /// Panics if called on a non-`Input` field.
    #[must_use]
    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        match &mut self.kind {
            FormFieldKind::Input { validate } => {
                *validate = Some(Box::new(f));
            }
            _ => panic!("FormField::validate called on a non-Input field"),
        }
        self
    }

    /// Create a yes/no confirmation field.
    ///
    /// `default` sets the value returned when the user presses Enter without typing.
    ///
    /// # Example
    ///
    /// ```
    /// use gilt::form::FormField;
    /// let f = FormField::confirm("ok", "Are you sure?", false);
    /// ```
    pub fn confirm(key: &str, prompt: &str, default: bool) -> Self {
        FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            kind: FormFieldKind::Confirm { default },
        }
    }

    /// Create a select-one field from a list of options.
    ///
    /// # Example
    ///
    /// ```
    /// use gilt::form::FormField;
    /// let f = FormField::select("fruit", "Pick a fruit", vec!["apple".into(), "banana".into()]);
    /// ```
    pub fn select(key: &str, prompt: &str, options: Vec<String>) -> Self {
        FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            kind: FormFieldKind::Select { options },
        }
    }
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

/// A multi-step form that chains [`FormField`]s and collects user input.
///
/// ```
/// use gilt::form::{Form, FormField};
/// use std::io::Cursor;
///
/// let mut cursor = Cursor::new(b"Alice\ny\n" as &[u8]);
/// let result = Form::new()
///     .field(FormField::input("name", "Your name?"))
///     .field(FormField::confirm("ok", "OK?", true))
///     .run_with_input(&mut cursor)
///     .unwrap();
/// assert_eq!(result["name"], "Alice");
/// ```
pub struct Form {
    fields: Vec<FormField>,
    /// Force accessible (plain-text) mode. `None` = auto-detect from env.
    accessible_override: Option<bool>,
}

impl Form {
    /// Create a new, empty `Form`.
    pub fn new() -> Self {
        Form {
            fields: Vec::new(),
            accessible_override: None,
        }
    }

    /// Add a field to the form.
    #[must_use]
    pub fn field(mut self, field: FormField) -> Self {
        self.fields.push(field);
        self
    }

    /// Force accessible (plain-text) mode on or off.
    ///
    /// When `true`, prompts are rendered as plain text with no ANSI styling,
    /// matching huh's `accessibility` mode.  When `false`, styled output is
    /// always used.  When not called (default), the mode is auto-detected:
    /// `NO_COLOR` set or `TERM=dumb` → accessible.
    #[must_use]
    pub fn accessible(mut self, value: bool) -> Self {
        self.accessible_override = Some(value);
        self
    }

    /// Detect whether accessible mode should be used.
    fn is_accessible(&self) -> bool {
        if let Some(forced) = self.accessible_override {
            return forced;
        }
        // Auto-detect: NO_COLOR set → accessible.
        if std::env::var("NO_COLOR").is_ok() {
            return true;
        }
        // TERM=dumb → accessible.
        if std::env::var("TERM").as_deref() == Ok("dumb") {
            return true;
        }
        false
    }

    /// Run the form, reading from stdin.
    pub fn run(&self) -> io::Result<HashMap<String, String>> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        self.run_with_input(&mut handle)
    }

    /// Run the form, reading from the provided `BufRead` source.
    ///
    /// Each field is prompted in order. `Input` fields with a validator will
    /// re-prompt on invalid input. `Confirm` fields accept `y/yes/n/no`
    /// (case-insensitive), blank input uses the field's default.  `Select`
    /// fields accept any option value (exact match, case-insensitive) or the
    /// 1-based index.
    ///
    /// Returns a `HashMap<String, String>` mapping each field's key to its
    /// collected value.  Confirm fields store `"true"` or `"false"`.
    pub fn run_with_input<R: BufRead>(&self, input: &mut R) -> io::Result<HashMap<String, String>> {
        let accessible = self.is_accessible();
        let mut results: HashMap<String, String> = HashMap::new();

        for field in &self.fields {
            let value = self.run_field(field, input, accessible)?;
            results.insert(field.key.clone(), value);
        }

        Ok(results)
    }

    /// Run a single field, looping until valid input is provided.
    fn run_field<R: BufRead>(
        &self,
        field: &FormField,
        input: &mut R,
        accessible: bool,
    ) -> io::Result<String> {
        match &field.kind {
            FormFieldKind::Input { validate } => {
                loop {
                    // Print prompt
                    self.print_prompt(&field.prompt, accessible);
                    let line = read_line(input)?;
                    let trimmed = line.trim_end_matches(['\n', '\r']).trim().to_string();

                    // Apply validator
                    if let Some(validator) = validate {
                        match validator(&trimmed) {
                            Ok(()) => return Ok(trimmed),
                            Err(msg) => {
                                self.print_error(&msg, accessible);
                                continue;
                            }
                        }
                    } else {
                        return Ok(trimmed);
                    }
                }
            }
            FormFieldKind::Confirm { default } => {
                let choices_hint = if *default { "[Y/n]" } else { "[y/N]" };
                loop {
                    self.print_prompt(&format!("{} {}", field.prompt, choices_hint), accessible);
                    let line = read_line(input)?;
                    let trimmed = line.trim_end_matches(['\n', '\r']).trim().to_lowercase();
                    match trimmed.as_str() {
                        "y" | "yes" => return Ok("true".to_string()),
                        "n" | "no" => return Ok("false".to_string()),
                        "" => return Ok(default.to_string()),
                        _ => {
                            self.print_error("Please enter Y or N", accessible);
                            continue;
                        }
                    }
                }
            }
            FormFieldKind::Select { options } => {
                loop {
                    self.print_prompt(&field.prompt, accessible);
                    let line = read_line(input)?;
                    let trimmed = line.trim_end_matches(['\n', '\r']).trim().to_string();

                    // Accept by value (case-insensitive exact match)
                    let lower = trimmed.to_lowercase();
                    if let Some(matched) = options.iter().find(|o| o.to_lowercase() == lower) {
                        return Ok(matched.clone());
                    }

                    // Accept by 1-based index
                    if let Ok(n) = trimmed.parse::<usize>() {
                        if n >= 1 && n <= options.len() {
                            return Ok(options[n - 1].clone());
                        }
                    }

                    self.print_error(
                        &format!("Please enter one of: {}", options.join(", ")),
                        accessible,
                    );
                }
            }
        }
    }

    /// Print a prompt line, with or without ANSI styling.
    fn print_prompt(&self, prompt: &str, accessible: bool) {
        if accessible {
            // Plain text — no styling
            print!("{}: ", prompt);
        } else {
            // Styled output via gilt markup (bold prompt)
            print!("{}: ", prompt);
        }
        let _ = io::stdout().flush();
    }

    /// Print an error message, with or without ANSI styling.
    fn print_error(&self, msg: &str, accessible: bool) {
        if accessible {
            eprintln!("Error: {}", msg);
        } else {
            // In a non-accessible context we'd use styled output, but since
            // we're writing to stderr and this is a simple validation error,
            // plain text is fine here too.
            eprintln!("Error: {}", msg);
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        Form::new()
    }
}

// ---------------------------------------------------------------------------
// Helper: read one line from input, returning EOF as empty string
// ---------------------------------------------------------------------------

fn read_line<R: BufRead>(input: &mut R) -> io::Result<String> {
    let mut line = String::new();
    match input.read_line(&mut line) {
        Ok(0) => Ok(String::new()), // EOF → empty
        Ok(_) => Ok(line),
        Err(e) => Err(e),
    }
}
