use crate::chunk_type::ChunkType;
use crate::{Error, Result};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fmt;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    pub chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self> {
        let mut length: [u8; 4] = [0; 4];
        let mut chunk_type: [u8; 4] = [0; 4];
        let mut crc: [u8; 4] = [0; 4];

        let mut z = 0;
        for &i in &value[0..=3] {
            length[z] = i;
            z += 1;
        }

        let mut t = 0;
        for &i in &value[4..=7] {
            chunk_type[t] = i;
            t += 1;
        }

        let v: Vec<_> = value
            .to_vec()
            .into_iter()
            .rev()
            .take(4)
            .rev()
            .map(|x| x)
            .collect();

        let mut p = 0;
        for i in v {
            crc[p] = i;
            p += 1;
        }

        let mut value = value.to_vec();

        value.pop();
        value.pop();
        value.pop();
        value.pop();

        value.remove(3);
        value.remove(2);
        value.remove(1);
        value.remove(0);

        let chunk_type: ChunkType = TryFrom::try_from(chunk_type).unwrap();

        if Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&value)
            != unsafe { std::mem::transmute::<[u8; 4], u32>(crc) }.to_be()
        {
            Err("crc error.".into())
        } else {
            value.remove(3);
            value.remove(2);
            value.remove(1);
            value.remove(0);

            Ok(Chunk {
                length: value.len() as u32,
                chunk_type,
                chunk_data: value,
                crc: unsafe { std::mem::transmute::<[u8; 4], u32>(crc) }.to_be(),
            })
        }
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {:?} {}",
            self.length, self.chunk_type, self.chunk_data, self.crc
        )
    }
}

#[allow(dead_code)]
impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let mut vec: Vec<u8> = Vec::new();
        for i in chunk_type.bytes() {
            vec.push(i);
        }
        for &i in &data {
            vec.push(i);
        }

        Chunk {
            length: data.len() as u32,
            chunk_type,
            chunk_data: data,
            crc: Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&vec),
        }
    }

    fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.chunk_data
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String> {
        let data = self.data().to_vec();
        let string = String::from_utf8(data).expect("Found invalid UTF-8");
        Ok(string)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        let length = unsafe { std::mem::transmute::<u32, [u8; 4]>(self.length.to_be()) };
        let chunk_type = [
            self.chunk_type.ancillary_byte,
            self.chunk_type.private_byte,
            self.chunk_type.reserved_byte,
            self.chunk_type.safe_to_copy_byte,
        ];
        let crc = unsafe { std::mem::transmute::<u32, [u8; 4]>(self.crc.to_be()) };

        for i in length {
            vec.push(i);
        }
        for i in chunk_type {
            vec.push(i);
        }
        for &i in &self.chunk_data {
            vec.push(i);
        }
        for i in crc {
            vec.push(i);
        }

        vec
    }

    pub fn read_chunk(reader: &mut BufReader<&[u8]>) -> Result<Chunk> {
        let mut buffer = [0; 4];

        reader.read_exact(&mut buffer)?;
        let length = u32::from_be_bytes(buffer);

        reader.read_exact(&mut buffer)?;
        let chunk_type = buffer.try_into()?;

        let mut chunk_data = vec![0; length as usize];
        reader.read_exact(&mut chunk_data)?;

        reader.read_exact(&mut buffer)?;
        let crc = u32::from_be_bytes(buffer);

        if crc != Self::crc_checksum(&chunk_type, &chunk_data) {
            return Err("invalid chunk".into());
        }

        Ok(Self {
            length,
            chunk_type,
            chunk_data,
            crc,
        })
    }

    fn crc_checksum(chunk_type: &ChunkType, data: &[u8]) -> u32 {
        let bytes: Vec<_> = chunk_type
            .bytes()
            .iter()
            .chain(data.iter())
            .copied()
            .collect();

        Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
