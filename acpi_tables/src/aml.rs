// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0
//

pub trait Aml {
    fn to_bytes(&self) -> Vec<u8>;
}

pub struct Path {
    root: bool,
    name_parts: Vec<[u8; 4]>,
}

impl Aml for Path {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        if self.root {
            bytes.push(b'\\');
        }

        match self.name_parts.len() {
            0 => panic!("Name cannot be empty"),
            1 => {}
            2 => {
                bytes.push(0x2e); /* DualNamePrefix */
            }
            n => {
                bytes.push(0x2f); /* MultiNamePrefix */
                bytes.push(n as u8);
            }
        };

        for part in self.name_parts.clone().iter_mut() {
            bytes.append(&mut part.to_vec());
        }

        bytes
    }
}

impl Path {
    pub fn new(name: &str) -> Self {
        let root = name.starts_with('\\');
        let offset = root as usize;
        let mut name_parts = Vec::new();
        for part in name[offset..].split('.') {
            assert_eq!(part.len(), 4);
            let mut name_part = [0u8; 4];
            name_part.copy_from_slice(part.as_bytes());
            name_parts.push(name_part);
        }

        Path { root, name_parts }
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Self {
        Path::new(s)
    }
}

pub type Byte = u8;

impl Aml for Byte {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x0a); /* BytePrefix */
        bytes.push(*self);
        bytes
    }
}

pub type Word = u16;

impl Aml for Word {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x0bu8); /* WordPrefix */
        bytes.append(&mut self.to_le_bytes().to_vec());
        bytes
    }
}

pub type DWord = u32;

impl Aml for DWord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x0c); /* DWordPrefix */
        bytes.append(&mut self.to_le_bytes().to_vec());
        bytes
    }
}

pub type QWord = u64;

impl Aml for QWord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x0e); /* QWordPrefix */
        bytes.append(&mut self.to_le_bytes().to_vec());
        bytes
    }
}

pub struct Name {
    bytes: Vec<u8>,
}

impl Aml for Name {
    fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

impl Name {
    pub fn new(path: Path, inner: &dyn Aml) -> Self {
        let mut bytes = Vec::new();
        bytes.push(0x08); /* NameOp */
        bytes.append(&mut path.to_bytes());
        bytes.append(&mut inner.to_bytes());
        Name { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_path() {
        assert_eq!(
            (&"_SB_".into() as &Path).to_bytes(),
            [0x5Fu8, 0x53, 0x42, 0x5F]
        );
        assert_eq!(
            (&"\\_SB_".into() as &Path).to_bytes(),
            [0x5C, 0x5F, 0x53, 0x42, 0x5F]
        );
        assert_eq!(
            (&"_SB_.COM1".into() as &Path).to_bytes(),
            [0x2E, 0x5F, 0x53, 0x42, 0x5F, 0x43, 0x4F, 0x4D, 0x31]
        );
        assert_eq!(
            (&"_SB_.PCI0._HID".into() as &Path).to_bytes(),
            [0x2F, 0x03, 0x5F, 0x53, 0x42, 0x5F, 0x50, 0x43, 0x49, 0x30, 0x5F, 0x48, 0x49, 0x44]
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(128u8.to_bytes(), [0x0a, 0x80]);
        assert_eq!(1024u16.to_bytes(), [0x0b, 0x0, 0x04]);
        assert_eq!((16u32 << 20).to_bytes(), [0x0c, 0x00, 0x00, 0x0, 0x01]);
        assert_eq!(
            0xdeca_fbad_deca_fbadu64.to_bytes(),
            [0x0e, 0xad, 0xfb, 0xca, 0xde, 0xad, 0xfb, 0xca, 0xde]
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(
            Name::new("_SB_.PCI0._UID".into(), &0x1234u16).to_bytes(),
            [
                0x08, /* NameOp */
                0x2F, /* MultiNamePrefix */
                0x03, /* 3 name parts */
                0x5F, 0x53, 0x42, 0x5F, /* _SB_ */
                0x50, 0x43, 0x49, 0x30, /* PCI0 */
                0x5F, 0x55, 0x49, 0x44, /* _UID  */
                0x0b, /* WordPrefix */
                0x34, 0x12
            ]
        );
    }
}