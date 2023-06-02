pub mod br_document;
pub mod cpf;
pub mod cnpj;

/// Enum that represents possible errors during validation of a document
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Indicates that the input string contained invalid characters.
    InvalidCharacters,
    /// Indicates that the provided document had a valid characters but did not pass validation.
    InvalidDocument,
    /// Indicates that the input was not valid and validation did not even occur.
    InvalidInput,
}

pub trait DocumentValidator : Sized {
    type Error;

    fn validate_input(input: &str) -> Option<Self::Error>;
}

pub(crate) fn to_integer_vector(input: &str) -> Vec<u32> {
    input.chars().map_while(|c| c.to_digit(10)).collect()
}

pub(crate) fn all_equal(n: &Vec<u32>) -> bool {
    n[1..].iter().try_fold(n[0], |prev, el| match prev {
        a if a == *el => Some(*el),
        _ => None,
    }).is_some()
}
