use std::{ops::Add, ffi::OsString};

use crate::MultipleVariant;

/// The file Properties
/// 
/// Default is both Data and Attributes
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum FileProperties {
    DATA,
    ATTRIBUTES,
    TIME_STAMPS,
    NTFS_ACCESS_CONTROL_LIST,
    OWNER_INFO,
    AUDITING_INFO,
    _MULTIPLE([bool; 6]),
}

impl Add for FileProperties {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_props = match self {
            Self::_MULTIPLE(props) => props,
            prop => {
                let mut val = 2_u8.pow(prop.index_of().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(props) => result_props = result_props.iter().zip(props.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            prop => result_props[prop.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_props)
    }
}

impl From<&FileProperties> for OsString {
    fn from(fp: &FileProperties) -> Self {
        let full ;
        OsString::from(match fp {
            FileProperties::DATA => "/copy:D",
            FileProperties::ATTRIBUTES => "/copy:A",
            FileProperties::TIME_STAMPS => "/copy:T",
            FileProperties::NTFS_ACCESS_CONTROL_LIST => "/copy:S",
            FileProperties::OWNER_INFO => "/copy:O",
            FileProperties::AUDITING_INFO => "/copy:U",
            FileProperties::_MULTIPLE(props) => {
                let part = ['D', 'A', 'T', 'S', 'O', 'U'].iter().zip(props.iter()).filter(|(_, exists)| **exists).unzip::<&char, &bool, String, Vec<bool>>().0;
                full = String::from("/copy:") + part.as_str();
                full.as_str()
            }
        })
    }
}
impl From<FileProperties> for OsString {
    fn from(fp: FileProperties) -> Self {
        (&fp).into()
    }
}

impl MultipleVariant for FileProperties {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(props) => {
                Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            prop => vec![*prop],
        }
    }
}

impl FileProperties {
    const VARIANTS: [Self; 6] = [
        Self::DATA,
        Self::ATTRIBUTES,
        Self::TIME_STAMPS,
        Self::NTFS_ACCESS_CONTROL_LIST,
        Self::OWNER_INFO,
        Self::AUDITING_INFO
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::DATA => Some(0),
            Self::ATTRIBUTES => Some(1),
            Self::TIME_STAMPS => Some(2),
            Self::NTFS_ACCESS_CONTROL_LIST => Some(3),
            Self::OWNER_INFO => Some(4),
            Self::AUDITING_INFO => Some(5),
            _ => None,
        }
    }

    /// Returns a variant containing all available file properties.
    #[allow(unused)]
    pub fn all() -> Self {
        Self::_MULTIPLE([true; 6])
    }

    /// Returns a variant containing no file properties.
    #[allow(unused)]
    pub fn none() -> Self {
        Self::_MULTIPLE([false; 6])
    }
}


/// The directory Properties
/// 
/// Default is both Data and Attributes
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum DirectoryProperties {
    DATA,
    ATTRIBUTES,
    TIME_STAMPS,
    _MULTIPLE([bool; 3])
}

impl Add for DirectoryProperties {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_props = match self {
            Self::_MULTIPLE(props) => props,
            prop => {
                let mut val = 2_u8.pow(prop.index_of().unwrap() as u32) + 2_u8; 
                (0..3).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(props) => result_props = result_props.iter().zip(props.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            prop => result_props[prop.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_props)
    }
}

impl From<&DirectoryProperties> for OsString {
    fn from(dp: &DirectoryProperties) -> Self {
        let full ;
        OsString::from(match dp {
            DirectoryProperties::DATA => "/dcopy:D",
            DirectoryProperties::ATTRIBUTES => "/dcopy:A",
            DirectoryProperties::TIME_STAMPS => "/dcopy:T",
            DirectoryProperties::_MULTIPLE(props) => {
                let part = ['D', 'A', 'T'].iter().zip(props.iter()).filter(|(_, exists)| **exists).unzip::<&char, &bool, String, Vec<bool>>().0;
                full = String::from("/dcopy:") + part.as_str();
                full.as_str()
            }
        })
    }
}
impl From<DirectoryProperties> for OsString {
    fn from(dp: DirectoryProperties) -> Self {
        (&dp).into()
    }
}

impl MultipleVariant for DirectoryProperties {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(props) => {
                Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            prop => vec![*prop],
        }
    }
}

impl DirectoryProperties {
    const VARIANTS: [Self; 3] = [
        Self::DATA,
        Self::ATTRIBUTES,
        Self::TIME_STAMPS,
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::DATA => Some(0),
            Self::ATTRIBUTES => Some(1),
            Self::TIME_STAMPS => Some(2),
            _ => None,
        }
    }

    /// Returns a variant containing all available directory properties.
    #[allow(unused)]
    pub fn all() -> Self {
        Self::_MULTIPLE([true; 3])
    }

    /// Returns a variant containing no directory properties.
    #[allow(unused)]
    pub fn none() -> Self {
        Self::_MULTIPLE([false; 3])
    }
}