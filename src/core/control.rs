use crate::common::data::Data;
use crate::core::extract::triop;

/// An implementation of an if statement, as an FFI.
/// Interesting idea, not sure if I'm going to keep it.
pub fn if_choice(data: Data) -> Result<Data, String> {
    if let (Data::Boolean(condition), option_a, option_b) = triop(data) {
        let choice = if condition { option_a } else { option_b };
        Ok(choice)
    } else {
        Err("\
            Expected the condition to be a boolean.\n\
            Note that Passerine does not have a notion of truthiness."
            .to_string())
    }
}
