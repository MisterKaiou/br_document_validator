use std::fmt::Display;
use crate::{all_equal, DocumentValidator, ErrorKind, to_integer_vector};

const CPF_SIZE: usize = 11;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CPFDocument(String);

impl TryFrom<String> for CPFDocument {
    type Error = ErrorKind;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CPFDocument::validate_input(&value)
            .map_or_else(
                || Ok(CPFDocument(value)),
                |err| Err(err))
    }
}

impl DocumentValidator for CPFDocument {
    type Error = ErrorKind;

    fn validate_input(input: &str) -> Option<Self::Error> {
        if input.len() != CPF_SIZE {
            return Some(ErrorKind::InvalidInput)
        }

        let input_as_integer_vector = to_integer_vector(input);

        if input_as_integer_vector.len() != CPF_SIZE {
            return Some(ErrorKind::InvalidCharacters)
        }
        if all_equal(&input_as_integer_vector) {
            return Some(ErrorKind::InvalidDocument)
        }

        validate_cpf(&input_as_integer_vector)
    }
}

impl Display for CPFDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn validate_cpf(numbers: &Vec<u32>) -> Option<ErrorKind> {
    fn digit_calculation(t: (u32, u32), curr: &u32) -> (u32, u32) {
        (t.0 + t.1 * curr, t.1 - 1)
    }
    fn ten_to_zero(n: u32) -> u32 {
        match n {
            10 => 0,
            _ => n,
        }
    }

    let first_nine_digits = &numbers[..9];

    let first_digit = ten_to_zero(first_nine_digits
            .iter()
            .fold((0, 10), digit_calculation).0 * 10 % 11
    );

    let (second_digit, curr) = first_nine_digits
        .iter()
        .fold((0, 11), digit_calculation);

    let second_digit = ten_to_zero((second_digit + first_digit * curr) * 10 % 11);

    if numbers[numbers.len() - 2] == first_digit && numbers[numbers.len() - 1] == second_digit {
        None
    } else {
        Some(ErrorKind::InvalidDocument)
    }
}
