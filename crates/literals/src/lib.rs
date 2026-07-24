use std::{
    fmt::Display,
    str::FromStr,
};

use num_bigint::{
    BigInt,
    BigUint,
};
use num_rational::BigRational;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Numeric(NumberLiteral),
    Text(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Numeric(n) => write!(f, "{n}"),
            Self::Text(t) => write!(f, "{t}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumberLiteral {
    pub value: NumberValue,
    pub suffix: Option<NumberSuffix>,
}

impl Display for NumberLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)?;
        if let Some(suffix) = &self.suffix {
            write!(f, "{suffix}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberValue {
    Integer(BigUint),
    Float(BigRational),
}

impl Display for NumberValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(fl) => write!(f, "{fl}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberSuffix {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl NumberSuffix {
    pub const fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

impl Display for NumberSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NumberSuffix {
    type Err = NumberLiteralParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "f32" => Ok(Self::F32),
            "f64" => Ok(Self::F64),
            _ => Err(NumberLiteralParsingError::InvalidSuffix(s.to_owned())),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NumberLiteralParsingError {
    #[error("empty number literal")]
    EmptyNumberLiteral,
    #[error("invalid number literal: {0}")]
    InvalidNumber(String),
    #[error("invalid digit in base-{radix} number literal: {literal}")]
    InvalidDigit { literal: String, radix: u32 },
    #[error("invalid digit separator in number literal: {0}")]
    InvalidSeparator(String),
    #[error("invalid number suffix: {0}")]
    InvalidSuffix(String),
    #[error("integer suffix `{0}` cannot be used on a floating-point literal")]
    IntegerSuffixOnFloat(NumberSuffix),
    #[error("invalid exponent: {0}")]
    InvalidExponent(String),
    #[error("number literal '{0}' is too large for the target type: {1}")]
    NumberTooLarge(String, NumberSuffix),
}

impl FromStr for NumberLiteral {
    type Err = NumberLiteralParsingError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.is_empty() {
            return Err(NumberLiteralParsingError::EmptyNumberLiteral);
        }
        if source.trim() != source {
            return Err(NumberLiteralParsingError::InvalidNumber(source.to_owned()));
        }

        let source = remove_separators(source)?;
        if source.starts_with('-') || source.starts_with('+') {
            return Err(NumberLiteralParsingError::InvalidNumber(source));
        }

        if let Some(rest) = source.strip_prefix("0x") {
            parse_based_integer(&source, rest, 16)
        } else if let Some(rest) = source.strip_prefix("0b") {
            parse_based_integer(&source, rest, 2)
        } else if let Some(rest) = source.strip_prefix("0o") {
            parse_based_integer(&source, rest, 8)
        } else {
            parse_decimal(&source)
        }
    }
}

fn remove_separators(source: &str) -> Result<String, NumberLiteralParsingError> {
    let bytes = source.as_bytes();
    for (index, &byte) in bytes.iter().enumerate() {
        if byte == b'_'
            && (index == 0
                || index + 1 == bytes.len()
                || bytes[index - 1] == b'_'
                || bytes[index + 1] == b'_')
        {
            return Err(NumberLiteralParsingError::InvalidSeparator(
                source.to_owned(),
            ));
        }
    }
    Ok(source.chars().filter(|&c| c != '_').collect())
}

fn parse_based_integer(
    source: &str,
    rest: &str,
    radix: u32,
) -> Result<NumberLiteral, NumberLiteralParsingError> {
    let digits_len = rest
        .char_indices()
        .take_while(|(_, c)| c.is_digit(radix))
        .last()
        .map_or(0, |(index, c)| index + c.len_utf8());
    let (digits, suffix_text) = rest.split_at(digits_len);

    if digits.is_empty() {
        return Err(NumberLiteralParsingError::InvalidDigit {
            literal: source.to_owned(),
            radix,
        });
    }
    if suffix_text.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(NumberLiteralParsingError::InvalidDigit {
            literal: source.to_owned(),
            radix,
        });
    }

    let suffix = parse_suffix(suffix_text)?;
    if suffix.is_some_and(NumberSuffix::is_float) {
        return Err(NumberLiteralParsingError::InvalidSuffix(
            suffix_text.to_owned(),
        ));
    }

    let value = BigUint::parse_bytes(digits.as_bytes(), radix).ok_or_else(|| {
        NumberLiteralParsingError::InvalidDigit {
            literal: source.to_owned(),
            radix,
        }
    })?;
    Ok(NumberLiteral {
        value: NumberValue::Integer(value),
        suffix,
    })
}

fn parse_decimal(source: &str) -> Result<NumberLiteral, NumberLiteralParsingError> {
    let bytes = source.as_bytes();
    let mut cursor = 0;

    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == 0 {
        return Err(NumberLiteralParsingError::InvalidNumber(source.to_owned()));
    }

    let mut is_float = false;
    if bytes.get(cursor) == Some(&b'.') {
        is_float = true;
        cursor += 1;
        let fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if cursor == fraction_start {
            return Err(NumberLiteralParsingError::InvalidNumber(source.to_owned()));
        }
    }

    if matches!(bytes.get(cursor), Some(b'e' | b'E')) {
        is_float = true;
        cursor += 1;
        if matches!(bytes.get(cursor), Some(b'+' | b'-')) {
            cursor += 1;
        }
        let exponent_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if cursor == exponent_start {
            return Err(NumberLiteralParsingError::InvalidExponent(
                source.to_owned(),
            ));
        }
    }

    let (numeric, suffix_text) = source.split_at(cursor);
    if !suffix_text.is_empty() && !suffix_text.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Err(NumberLiteralParsingError::InvalidNumber(source.to_owned()));
    }
    let suffix = parse_suffix(suffix_text)?;

    if is_float {
        if let Some(suffix) = suffix
            && !suffix.is_float()
        {
            return Err(NumberLiteralParsingError::IntegerSuffixOnFloat(suffix));
        }
        Ok(NumberLiteral {
            value: NumberValue::Float(decimal_to_rational(numeric)?),
            suffix,
        })
    } else if suffix.is_some_and(NumberSuffix::is_float) {
        let integer = parse_biguint(numeric, 10)?;
        Ok(NumberLiteral {
            value: NumberValue::Float(BigRational::from_integer(BigInt::from(integer))),
            suffix,
        })
    } else {
        Ok(NumberLiteral {
            value: NumberValue::Integer(parse_biguint(numeric, 10)?),
            suffix,
        })
    }
}

fn parse_suffix(suffix: &str) -> Result<Option<NumberSuffix>, NumberLiteralParsingError> {
    if suffix.is_empty() {
        Ok(None)
    } else {
        suffix.parse().map(Some)
    }
}

fn parse_biguint(source: &str, radix: u32) -> Result<BigUint, NumberLiteralParsingError> {
    BigUint::parse_bytes(source.as_bytes(), radix).ok_or_else(|| {
        NumberLiteralParsingError::InvalidDigit {
            literal: source.to_owned(),
            radix,
        }
    })
}

fn decimal_to_rational(source: &str) -> Result<BigRational, NumberLiteralParsingError> {
    let (mantissa, exponent) = match source.find(['e', 'E']) {
        Some(index) => {
            let exponent = source[index + 1..]
                .parse::<i64>()
                .map_err(|_| NumberLiteralParsingError::InvalidExponent(source.to_owned()))?;
            (&source[..index], exponent)
        }
        None => (source, 0),
    };

    let (integer, fraction) = match mantissa.split_once('.') {
        Some(parts) => parts,
        None => (mantissa, ""),
    };
    let digits = format!("{integer}{fraction}");
    let mut numerator = parse_biguint(&digits, 10)?;
    let decimal_places = i64::try_from(fraction.len())
        .map_err(|_| NumberLiteralParsingError::InvalidExponent(source.to_owned()))?;
    let scale = decimal_places
        .checked_sub(exponent)
        .ok_or_else(|| NumberLiteralParsingError::InvalidExponent(source.to_owned()))?;

    if scale >= 0 {
        let scale = u32::try_from(scale)
            .map_err(|_| NumberLiteralParsingError::InvalidExponent(source.to_owned()))?;
        let denominator = BigUint::from(10u8).pow(scale);
        Ok(BigRational::new(
            BigInt::from(numerator),
            BigInt::from(denominator),
        ))
    } else {
        let scale = u32::try_from(-scale)
            .map_err(|_| NumberLiteralParsingError::InvalidExponent(source.to_owned()))?;
        numerator *= BigUint::from(10u8).pow(scale);
        Ok(BigRational::from_integer(BigInt::from(numerator)))
    }
}
