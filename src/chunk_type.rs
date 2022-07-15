use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct ChunkTypeError<'a>(&'a str);

// Error doesn't require you to implement any methods, but
// your type must also implement Debug and Display.
impl<'a> Error for ChunkTypeError<'a> {}

impl<'a> fmt::Display for ChunkTypeError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(PartialEq, Debug, Eq)]
pub struct ChunkType {
    pub ancillary_byte: u8,
    pub private_byte: u8,
    pub reserved_byte: u8,
    pub safe_to_copy_byte: u8,
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chunk_type_u8 = [
            self.ancillary_byte,
            self.private_byte,
            self.reserved_byte,
            self.safe_to_copy_byte,
        ];
        let c = std::str::from_utf8(&chunk_type_u8).unwrap();
        write!(f, "{}", c)
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeError<'static>;
    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        for i in value {
            if i < 65 {
                return Err(ChunkTypeError(
                    "Error! The value array cannot be parsed as letters.",
                ));
            }
            if i > 122 {
                return Err(ChunkTypeError(
                    "Error! The value array cannot be parsed as letters.",
                ));
            }
            if i > 90 && i < 97 {
                return Err(ChunkTypeError(
                    "Error! The value array cannot be parsed as letters.",
                ));
            }
        }

        Ok(ChunkType {
            ancillary_byte: value[0],
            private_byte: value[1],
            reserved_byte: value[2],
            safe_to_copy_byte: value[3],
        })
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeError<'static>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();
        for &i in s {
            if i < 65 {
                return Err(ChunkTypeError(
                    "Error! The str cannot be parsed as letters.",
                ));
            }
            if i > 122 {
                return Err(ChunkTypeError(
                    "Error! The str cannot be parsed as letters.",
                ));
            }
            if i > 90 && i < 97 {
                return Err(ChunkTypeError(
                    "Error! The str cannot be parsed as letters.",
                ));
            }
        }

        Ok(ChunkType {
            ancillary_byte: s[0],
            private_byte: s[1],
            reserved_byte: s[2],
            safe_to_copy_byte: s[3],
        })
    }
}

#[allow(dead_code)]
impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        [
            self.ancillary_byte,
            self.private_byte,
            self.reserved_byte,
            self.safe_to_copy_byte,
        ]
    }

    fn is_valid(&self) -> bool {
        if !self.is_reserved_bit_valid() {
            return false;
        }
        true
    }

    fn is_critical(&self) -> bool {
        (self.ancillary_byte & 32) as u8 == 0
    }

    fn is_public(&self) -> bool {
        (self.private_byte & 32) as u8 == 0
    }

    fn is_reserved_bit_valid(&self) -> bool {
        (self.reserved_byte & 32) as u8 == 0
    }

    fn is_safe_to_copy(&self) -> bool {
        (self.safe_to_copy_byte & 32) as u8 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
