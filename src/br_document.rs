use std::{convert::TryFrom, str::FromStr, fmt::Display};
use crate::cpf::CPFDocument;
use crate::{DocumentValidator, ErrorKind};
use crate::cnpj::CNPJDocument;
use crate::br_document::DocumentNumber::{CNPJ, CPF};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum DocumentNumber {
    CPF(CPFDocument),
    CNPJ(CNPJDocument),
}

impl TryFrom<String> for DocumentNumber {
    type Error = ErrorKind;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CPFDocument::try_from(value.clone())
            .map(|it| CPF(it))
            .or_else(|error| {
                match error {
                    ErrorKind::InvalidCharacters | ErrorKind::InvalidDocument => Err(error),
                    ErrorKind::InvalidInput => Ok(CNPJ(CNPJDocument::try_from(value)?))
                }
            })
    }
}

impl FromStr for DocumentNumber {
    type Err = ErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DocumentNumber::try_from(s.to_string())
    }
}

impl Display for DocumentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            CPF(s) => s.to_string(),
            CNPJ(s) => s.to_string()
        })
    }
}

impl DocumentValidator for DocumentNumber {
    type Error = ErrorKind;

    fn validate_input(input: &str) -> Option<Self::Error> {
        if let Some(error) = CPFDocument::validate_input(input) {
            return match error {
                ErrorKind::InvalidCharacters | ErrorKind::InvalidDocument => Some(error),
                ErrorKind::InvalidInput => CNPJDocument::validate_input(input)
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{DocumentNumber, ErrorKind};
    use test_case::test_case;
    use crate::DocumentValidator;

    #[test_case("96865090039", None                                     ; "Valid CPFs should be allowed")]
    #[test_case("03165685000114", None                                  ; "Valid CNPJ should be allowed")]
    #[test_case("11111111111", Some(ErrorKind::InvalidDocument)         ; "All equal characters CPF should not be allowed")]
    #[test_case("79888245131", Some(ErrorKind::InvalidDocument)         ; "CPF with incorrect verification digits should not be allowed")]
    #[test_case("73361907000130", Some(ErrorKind::InvalidDocument)      ; "CNPJ with incorrect verification digits should not be allowed")]
    #[test_case("272676S6021", Some(ErrorKind::InvalidCharacters)       ; "CPF with invalid characters should not be allowed")]
    #[test_case("896S4922000126", Some(ErrorKind::InvalidCharacters)    ; "CNPJ with invalid characters should not be allowed")]
    #[test_case("6611493500107", Some(ErrorKind::InvalidInput)          ; "Input with incorrect character count should not be allowed - 1")]
    #[test_case("2881121027", Some(ErrorKind::InvalidInput)             ; "Input with incorrect character count should not be allowed - 2")]
    #[test_case("661149350000107", Some(ErrorKind::InvalidInput)        ; "Input with incorrect character count should not be allowed - 3")]
    #[test_case("288111221027", Some(ErrorKind::InvalidInput)           ; "Input with incorrect character count should not be allowed - 4")]
    fn validate(input: &str, expected: Option<ErrorKind>) {
        let actual = DocumentNumber::validate_input(&input);

        assert_eq!(actual, expected);
    }
}
