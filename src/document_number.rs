use lazy_static::lazy_static;
use regex::Regex;
use std::{convert::TryFrom, str::FromStr, fmt::Display};
use Vec;

lazy_static! {
    static ref CPF_MASK: Regex = Regex::new(r"^\d{3}[.]\d{3}[.]\d{3}[-]\d{2}$").unwrap();
    static ref CNPJ_MASK: Regex = Regex::new(r"^\d{2}[.]\d{3}[.]\d{3}[\/]\d{4}[-]\d{2}$").unwrap();
}

const CNPJ_POSITIONAL_WEIGHTS: [u32; 13] = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
const SANITIZED_CPF_SIZE: usize = 11;
const FORMATTED_CPF_AND_SANITIZED_CNPJ_SIZE: usize = 14;
const FORMATTED_CNPJ_SIZE: usize = 18;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CPFDocument(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CNPJDocument(String);

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Indicates that the input string had invalid characters.
    InvalidCharacters,
    /// Indicates that the given document had a valid format but did not pass validation.
    InvalidDocument,
    /// Indicates that the input was not valid, and validation did not even started.
    InvalidInput,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum DocumentNumber {
    CPF(CPFDocument),
    CNPJ(CNPJDocument),
}

impl TryFrom<String> for DocumentNumber {
    type Error = ErrorKind;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl FromStr for DocumentNumber {
    type Err = ErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s.to_string())
    }
}

impl Display for DocumentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DocumentNumber::CPF(s) => s.to_string(),
            DocumentNumber::CNPJ(s) => s.to_string()
        })
    }
}

impl Display for CPFDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for CNPJDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DocumentNumber {
    pub fn validate(input: &str) -> Result<(), ErrorKind> {
        DocumentNumber::evaluate(input).map(|_| ())
    }

    pub fn parse(input: String) -> Result<DocumentNumber, ErrorKind> {
        match DocumentNumber::evaluate(&input)? {
            n if n.len() == SANITIZED_CPF_SIZE => {
                Ok(DocumentNumber::CPF(CPFDocument(numbers_to_string(n))))
            }
            n => Ok(DocumentNumber::CNPJ(CNPJDocument(numbers_to_string(n)))),
        }
    }

    fn evaluate(input: &str) -> Result<Vec<u32>, ErrorKind> {
        match input.len() {
            SANITIZED_CPF_SIZE => match to_integer_vector(input) {
                n if n.len() == SANITIZED_CPF_SIZE && !all_equal(&n) => validate_cpf(&n).map(|_| n),
                n if n.len() != SANITIZED_CPF_SIZE => Err(ErrorKind::InvalidCharacters),
                _ => Err(ErrorKind::InvalidDocument),
            },
            FORMATTED_CPF_AND_SANITIZED_CNPJ_SIZE => {
                match CPF_MASK.is_match(&input) {
                    true => match to_integer_vector_sanitizing(input) {
                        n if !all_equal(&n) => validate_cpf(&n).map(|_| n),
                        _ => Err(ErrorKind::InvalidDocument)
                    },
                    false => match to_integer_vector(input) {
                        n if n.len() == FORMATTED_CPF_AND_SANITIZED_CNPJ_SIZE => validate_cnpj(&n).map(|_| n),
                        _ => Err(ErrorKind::InvalidCharacters),
                    },
                }
            }
            FORMATTED_CNPJ_SIZE => {
                match CNPJ_MASK.is_match(&input) {
                    true => match to_integer_vector_sanitizing(input) {
                        n if n.len() == FORMATTED_CPF_AND_SANITIZED_CNPJ_SIZE => validate_cnpj(&n).map(|_| n),
                        _ => Err(ErrorKind::InvalidCharacters),
                    },
                    false => Err(ErrorKind::InvalidCharacters)
                }
            }
            _ => Err(ErrorKind::InvalidInput),
        }
    }
}

fn to_integer_vector_sanitizing(input: &str) -> Vec<u32> {
    input.chars().filter_map(|c| c.to_digit(10)).collect()
}

fn to_integer_vector(input: &str) -> Vec<u32> {
    input.chars().map_while(|c| c.to_digit(10)).collect()
}

fn validate_cpf(numbers: &Vec<u32>) -> Result<(), ErrorKind> {
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

    let first_digit =
        ten_to_zero(first_nine_digits.iter().fold((0, 10), digit_calculation).0 * 10 % 11);

    let (second_digit, curr) = first_nine_digits.iter().fold((0, 11), digit_calculation);

    let second_digit = ten_to_zero((second_digit + first_digit * curr) * 10 % 11);

    if check_digits(
        numbers[numbers.len() - 2],
        numbers[numbers.len() - 1],
        first_digit,
        second_digit,
    ) {
        Ok(())
    } else {
        Err(ErrorKind::InvalidDocument)
    }
}

fn validate_cnpj(numbers: &Vec<u32>) -> Result<(), ErrorKind> {
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

    if check_digits(
        numbers[numbers.len() - 2],
        numbers[numbers.len() - 1],
        first_digit,
        second_digit,
    ) {
        Ok(())
    } else {
        Err(ErrorKind::InvalidDocument)
    }
}

fn numbers_to_string(n: Vec<u32>) -> String {
    n.iter().map(|n| char::from_digit(*n, 10).unwrap()).collect()
}

fn check_digits(
    actual_first_digit: u32,
    actual_second_digit: u32,
    expected_first_digit: u32,
    expected_second_digit: u32,
) -> bool {
    actual_first_digit == expected_first_digit && actual_second_digit == expected_second_digit
}

fn all_equal(n: &Vec<u32>) -> bool {
    n[1..].iter().try_fold(n[0], |prev, el| match prev {
        a if a == *el => Some(*el),
        _ => None,
    }).is_some()
}

#[cfg(test)]
mod tests {
    use super::{DocumentNumber, ErrorKind};
    use test_case::test_case;

    #[test_case("96865090039", Ok(())                                   ; "Sanitized CPF should be allowed")]
    #[test_case("288.111.210-27", Ok(())                                ; "Formatted CPF should be allowed")]
    #[test_case("03165685000114", Ok(())                                ; "Sanitized CNPJ should be allowed")]
    #[test_case("89.654.922/0001-26", Ok(())                            ; "Formatted CNPJ should be allowed")]
    #[test_case("111.111.111-11", Err(ErrorKind::InvalidDocument)       ; "Formatted all equal characters CPF should not be allowed")]
    #[test_case("11111111111", Err(ErrorKind::InvalidDocument)          ; "Sanitized all equal characters CPF should not be allowed")]
    #[test_case("798.882.451-31", Err(ErrorKind::InvalidDocument)       ; "CPF with incorrect verification digits should not be allowed")]
    #[test_case("73.361.907/0001-30", Err(ErrorKind::InvalidDocument)   ; "CNPJ with incorrect verification digits should not be allowed")]
    #[test_case("272.676.S60-21", Err(ErrorKind::InvalidCharacters)     ; "Formatted CPF with invalid characters should not be allowed")]
    #[test_case("272676S6021", Err(ErrorKind::InvalidCharacters)        ; "Sanitized CPF with invalid characters should not be allowed")]
    #[test_case("272-676.560-21", Err(ErrorKind::InvalidCharacters)     ; "CPF with invalid formatting should not be allowed")]
    #[test_case("66.114.93S/0001-07", Err(ErrorKind::InvalidCharacters) ; "Sanitized CNPJ with invalid characters should not be allowed")]
    #[test_case("896S4922000126", Err(ErrorKind::InvalidCharacters)     ; "Formatted CNPJ with invalid characters should not be allowed")]
    #[test_case("66.114-935/0001-07", Err(ErrorKind::InvalidCharacters) ; "CNPJ with invalid formatting should not be allowed")]
    #[test_case("66.114-935/001-07", Err(ErrorKind::InvalidInput)       ; "Input with incorrect character count should not be allowed - 1")]
    #[test_case("288.11.210-27", Err(ErrorKind::InvalidInput)           ; "Input with incorrect character count should not be allowed - 2")]
    #[test_case("66.114-935/00001-07", Err(ErrorKind::InvalidInput)     ; "Input with incorrect character count should not be allowed - 3")]
    #[test_case("288.1112.210-27", Err(ErrorKind::InvalidInput)         ; "Input with incorrect character count should not be allowed - 4")]
    fn validate(input: &str, expected: Result<(), ErrorKind>) {
        let actual = DocumentNumber::validate(&input.to_string());

        assert_eq!(actual, expected);
    }
}
