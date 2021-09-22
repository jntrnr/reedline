/// The syntax validation trait. Implementers of this trait will check to see if the current input
/// is incomplete and spans multiple lines
pub trait Validator {
    /// The action that will handle the current buffer as a line and return the corresponding validation
    fn validate(&self, line: &str) -> ValidationResult;
}

/// Whether or not the validation shows the input was complete
pub enum ValidationResult {
    /// An incomplete input which may need to span multiple lines to be complete
    Incomplete,

    /// An input that is complete as-is
    Complete,
}

/// A default validator which checks for mismatched quotes
pub struct DefaultValidator;

impl Validator for DefaultValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.split('"').count() % 2 == 0 {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}
