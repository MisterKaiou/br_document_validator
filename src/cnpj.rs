use std::fmt::Display;
use crate::{all_equal, DocumentValidator, ErrorKind, to_integer_vector};

const CNPJ_POSITIONAL_WEIGHTS: [u32; 13] = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
const CNPJ_SIZE: usize = 14;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CNPJDocument(String);

impl TryFrom<String> for CNPJDocument {
    type Error = ErrorKind;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CNPJDocument::validate_input(&value)
            .map_or(
                Ok(CNPJDocument(value.to_string())),
                |err| Err(err))
    }
}

impl DocumentValidator for CNPJDocument {
    type Error = ErrorKind;

    fn validate_input(input: &str) -> Option<Self::Error> {
        if input.len() != CNPJ_SIZE {
            return Some(ErrorKind::InvalidInput)
        }

        let input_as_integer_vector = to_integer_vector(input);

        if input_as_integer_vector.len() != CNPJ_SIZE {
            return Some(ErrorKind::InvalidCharacters)
        }
        if all_equal(&input_as_integer_vector) {
            return Some(ErrorKind::InvalidDocument)
        }

        validate_cnpj(&input_as_integer_vector)
    }
}

impl Display for CNPJDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn validate_cnpj(numbers: &Vec<u32>) -> Option<ErrorKind> {
    fn calculate_digit(subject: u32) -> u32 {
        match subject % 11 {
            r if r < 2 => 0,
            r @ _ => 11 - r,
        }
    }

    let first_twelve_digits = &numbers[..12];
    let first_zipped_sum: u32 = first_twelve_digits
        .iter()
        .zip(&CNPJ_POSITIONAL_WEIGHTS[1..])
        .map(|t| t.0 * t.1)
        .sum();

    let first_digit = calculate_digit(first_zipped_sum);

    let second_zipped_sum = [first_twelve_digits, &[first_digit]]
        .concat()
        .iter()
        .zip(CNPJ_POSITIONAL_WEIGHTS.iter())
        .map(|t| t.0 * t.1)
        .sum();

    let second_digit = calculate_digit(second_zipped_sum);

    if numbers[numbers.len() - 2] == first_digit && numbers[numbers.len() - 1] == second_digit {
        None
    } else {
        Some(ErrorKind::InvalidDocument)
    }
}