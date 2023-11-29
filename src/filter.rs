//! Handle for Robocopy file and directory filter options
//! 
//! All filters and exceptions are handled by the Filter struct

use std::{convert::TryInto, ffi::OsString, ops::Add};
use crate::FileAttributes;
use crate::MultipleVariant;

/// Filters out files that match the variant
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum FileExclusionFilter {
    /// Excludes files for which any of the specified attributes are set. Corresponds to `/xa` option.
    Attributes(FileAttributes),
    /// Excludes files that match the specified names or paths. Wildcard characters (* and ?) are supported. Corresponds to `/xf` option.
    PathOrName(Vec<String>),
    /// Excludes existing files with the same timestamp, but different file sizes. Corresponds to `/xc` option.
    CHANGED,
    /// Source directory files older than the destination are excluded from the copy. Corresponds to `/xo` option.
    OLDER,
    /// Source directory files newer than the destination are excluded from the copy. Corresponds to `/xn` option.
    NEWER,
    /// Excludes junction points for files. Corresponds to `/xjf` option.
    JUNCTION_POINTS,
    _MULTIPLE(Option<FileAttributes>, Vec<String>, [bool; 4])
}

impl Add for FileExclusionFilter {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let (mut result_attribs, mut result_path_or_name, mut result_filters) = match self {
            Self::_MULTIPLE(attribs, path_or_name, filters) => (attribs, path_or_name, filters),
            Self::Attributes(attribs) => (Some(attribs), Vec::new(), [false; 4]),
            Self::PathOrName(path_or_name) => (None, path_or_name, [false; 4]),
            filter => {
                let mut val = 2_u8.pow(filter.index_of().unwrap() as u32) + 2_u8; 
                (None, Vec::new(), (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap())
            }
        };

        match rhs {
            Self::_MULTIPLE(attribs, mut path_or_name, filters) => {
                result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap();
                if let Some(attribs) = attribs {
                    result_attribs = match result_attribs {
                        Some(res_attribs) => Some(attribs + res_attribs),
                        None => Some(attribs)
                    };
                }
                result_path_or_name.append(&mut path_or_name);
            },
            Self::Attributes(attribs) => result_attribs = match result_attribs {
                Some(res_attribs) => Some(attribs + res_attribs),
                None => Some(attribs)
            },
            Self::PathOrName(mut path_or_name) => result_path_or_name.append(&mut path_or_name),
            filter => result_filters[filter.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_attribs, result_path_or_name, result_filters)
    }
}

impl MultipleVariant for FileExclusionFilter {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(attribs, path_or_name, props) => {
                let mut filters: Vec<FileExclusionFilter> = Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).map(|(variant, _)| variant.clone() ).collect();
                
                if let Some(attribs) = attribs {
                    filters.push(Self::Attributes(*attribs));
                }

                if !path_or_name.is_empty() {
                    filters.push(Self::PathOrName(path_or_name.clone()))
                }

                filters
            },
            prop => vec![prop.clone()],
        }
    }
}

impl From<&FileExclusionFilter> for Vec<OsString> {
    fn from(fef: &FileExclusionFilter) -> Self {
        let mut res = Vec::new();
        fef.single_variants().iter().for_each(|filter| match filter {
            FileExclusionFilter::Attributes(file_attributes) => res.push(OsString::from(String::from("/xa:") + Into::<OsString>::into(file_attributes).to_str().unwrap())),
            FileExclusionFilter::PathOrName(path_or_name) => {
                res.push(OsString::from("/xf"));
                path_or_name.iter().for_each(|path_or_name| res.push(OsString::from(path_or_name.as_str())));
            },
            FileExclusionFilter::CHANGED => res.push(OsString::from("/xc")),
            FileExclusionFilter::OLDER => res.push(OsString::from("/xo")),
            FileExclusionFilter::NEWER => res.push(OsString::from("/xn")),
            FileExclusionFilter::JUNCTION_POINTS => res.push(OsString::from("/xjf")),
            _ => unreachable!()
        });
        res
    }
}
impl From<FileExclusionFilter> for Vec<OsString> {
    fn from(fef: FileExclusionFilter) -> Self {
        (&fef).into()
    }
}

impl FileExclusionFilter {
    const VARIANTS: [Self; 4] = [
        Self::CHANGED,
        Self::OLDER,
        Self::NEWER,
        Self::JUNCTION_POINTS
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::CHANGED => Some(0),
            Self::NEWER => Some(2),
            Self::JUNCTION_POINTS => Some(3),
            _ => None,
        }
    }
}

/// Filters out directories that match the variant
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum DirectoryExclusionFilter {
    /// Excludes directories that match the specified names and paths. Corresponds to `/xd` option.
    PathOrName(Vec<String>),
    /// Excludes junction points for directories. Corresponds to `/xjd` option.
    JUNCTION_POINTS,
    _BOTH(Vec<String>)
}

impl Add for DirectoryExclusionFilter {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut junction_pts = false;

        let mut result_path_or_name = match self {
            Self::PathOrName(attribs) | Self::_BOTH(attribs) => attribs,
            Self::JUNCTION_POINTS => { junction_pts = true; Vec::new() }
        };

        match rhs {
            Self::PathOrName(mut attribs) | Self::_BOTH(mut attribs) => result_path_or_name.append(&mut attribs),
            _ => junction_pts = true
        };

        if junction_pts {
            Self::_BOTH(result_path_or_name)
        } else {
            Self::PathOrName(result_path_or_name)
        }
    }
}

impl From<&DirectoryExclusionFilter> for Vec<OsString> {
    fn from(def: &DirectoryExclusionFilter) -> Self {
        let mut res = Vec::new();
        def.single_variants().iter().for_each(|filter| match filter {
            DirectoryExclusionFilter::PathOrName(path_or_name) => {
                res.push(OsString::from("/xd"));
                path_or_name.iter().for_each(|path_or_name| res.push(OsString::from(path_or_name.as_str())));
            },
            DirectoryExclusionFilter::JUNCTION_POINTS => res.push(OsString::from("/xjd")),
            _ => unreachable!()
        });
        res
    }
}
impl From<DirectoryExclusionFilter> for Vec<OsString> {
    fn from(def: DirectoryExclusionFilter) -> Self {
        (&def).into()
    }
}

impl MultipleVariant for DirectoryExclusionFilter {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_BOTH(path_or_name) => vec![Self::JUNCTION_POINTS, Self::PathOrName(path_or_name.clone())],
            Self::JUNCTION_POINTS => vec![Self::JUNCTION_POINTS],
            Self::PathOrName(path_or_name) => vec![Self::PathOrName(path_or_name.clone())]
        }
    }
}


/// Filters out files and directories that match the variant
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum FileAndDirectoryExclusionFilter {
    /// Excludes extra files and directories present in the destination but not the source.
    /// 
    /// Excluding extra files won't delete files from the destination.
    /// 
    /// Corresponds to `/xx` option.
    EXTRA,
    /// Excludes "lonely" files and directories present in the source but not the destination.
    /// 
    /// Excluding lonely files prevents any new files from being added to the destination.
    /// 
    /// Corresponds to `/xl` option.
    LONELY,
    /// Excludes junction points, which are normally included by default.
    /// 
    /// Corresponds to `/xj` option.
    JUNCTION_POINTS,
    _MULTIPLE([bool; 3])
}

impl Add for FileAndDirectoryExclusionFilter {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_filters = match self {
            Self::_MULTIPLE(filters) => filters,
            filter => {
                let mut val = 2_u8.pow(filter.index_of().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(filters) => result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            filter => result_filters[filter.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_filters)
    }
}

impl From<&FileAndDirectoryExclusionFilter> for Vec<OsString> {
    fn from(fadef: &FileAndDirectoryExclusionFilter) -> Self {
        let mut res = Vec::new();
        fadef.single_variants().iter().for_each(|filter| match filter {
            FileAndDirectoryExclusionFilter::EXTRA => res.push(OsString::from("/xx")),
            FileAndDirectoryExclusionFilter::LONELY => res.push(OsString::from("/xl")),
            FileAndDirectoryExclusionFilter::JUNCTION_POINTS => res.push(OsString::from("/xj")),
            _ => unreachable!()
        });
        res
    }
}
impl From<FileAndDirectoryExclusionFilter> for Vec<OsString> {
    fn from(fadef: FileAndDirectoryExclusionFilter) -> Self {
        (&fadef).into()
    }
}

impl MultipleVariant for FileAndDirectoryExclusionFilter {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(filters) => {
                Self::VARIANTS.iter().zip(filters.iter()).filter(|(_, exists)| **exists).unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
    }
}

impl FileAndDirectoryExclusionFilter {
    const VARIANTS: [Self; 3] = [
        Self::EXTRA,
        Self::LONELY,
        Self::JUNCTION_POINTS
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::EXTRA => Some(0),
            Self::LONELY => Some(1),
            Self::JUNCTION_POINTS => Some(2),
            _ => None,
        }
    }
}

/// Includes files despite the filters that match the variant
#[derive(Debug, Copy, Clone)]
pub enum FileExclusionFilterException {
    /// Include modified files (differing change times).
    /// 
    /// Corresponds to `/im` option.
    MODIFIED,
    /// Includes the same files. Same files are identical in name, size, times, and all attributes.
    /// 
    /// Corresponds to `/is` option.
    SAME,
    /// Includes "tweaked" files. Tweaked files have the same name, size, and times, but different attributes.
    /// 
    /// Corresponds to `/it` option.
    TWEAKED,
    _MULTIPLE([bool; 3])
}

impl Add for FileExclusionFilterException {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_filters = match self {
            Self::_MULTIPLE(filters) => filters,
            filter => {
                let mut val = 2_u8.pow(filter.index_of().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(filters) => result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            filter => result_filters[filter.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_filters)
    }
}

impl From<&FileExclusionFilterException> for Vec<OsString> {
    fn from(fefe: &FileExclusionFilterException) -> Self {
        let mut res = Vec::new();
        fefe.single_variants().iter().for_each(|filter| match filter {
            FileExclusionFilterException::MODIFIED => res.push(OsString::from("/im")),
            FileExclusionFilterException::SAME => res.push(OsString::from("/is")),
            FileExclusionFilterException::TWEAKED => res.push(OsString::from("/it")),
            _ => unreachable!()
        });
        res
    }
}
impl From<FileExclusionFilterException> for Vec<OsString> {
    fn from(fefe: FileExclusionFilterException) -> Self {
        (&fefe).into()
    }
}

impl MultipleVariant for FileExclusionFilterException {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(filters) => {
                Self::VARIANTS.iter().zip(filters.iter()).filter(|(_, exists)| **exists).unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
    }
}

impl FileExclusionFilterException {
    const VARIANTS: [Self; 3] = [
        Self::MODIFIED,
        Self::SAME,
        Self::TWEAKED
    ];

    /// Returns the index of the variant in a 
    /// FileExclusionFilterException::_MULTIPLE variant
    /// and the Self::VARIANTS array
    fn index_of(&self) -> Option<usize>{
        match self {
            Self::MODIFIED => Some(0),
            Self::SAME => Some(1),
            Self::TWEAKED => Some(2),
            _ => None,
        }
    }
}

/// Handles all filter attributes supported by Robocopy
#[derive(Debug, Clone, Default)]
pub struct Filter<'a> {
    /// Copies only files for which the Archive attribute is set, and resets the Archive attribute.
    /// 
    /// Corresponds to `/m` option.
    pub handle_archive_and_reset: bool,

    /// Includes only files for which any of the specified attributes are set.
    /// 
    /// Corresponds to `/ia` option.
    pub include_only_files_with_any_of_these_attribs: Option<FileAttributes>,

    /// Filters out which files to copy.
    pub file_exclusion_filter: Option<FileExclusionFilter>,
    /// Filters out which directories to copy.
    pub directory_exclusion_filter: Option<DirectoryExclusionFilter>,
    /// Filters out which files and directories to copy.
    pub file_and_directory_exclusion_filter: Option<FileAndDirectoryExclusionFilter>,
    /// Includes files despite the filters.
    pub file_exclusion_filter_exceptions: Option<FileExclusionFilterException>,

    /// Specifies the maximum file size (to exclude files bigger than n bytes).
    /// 
    /// Corresponds to `/max` option.
    pub max_size: Option<u128>,
    /// Specifies the minimum file size (to exclude files smaller than n bytes).
    /// 
    /// Corresponds to `/min` option.
    pub min_size: Option<u128>,

    /// Specifies the maximum file age (to exclude files older than n days or date).
    /// 
    /// Corresponds to `/maxage` option.
    pub max_age: Option<&'a str>,
    /// Specifies the minimum file age (exclude files newer than n days or date).
    /// 
    /// Corresponds to `/minage` option.
    pub min_age: Option<&'a str>,

    /// Specifies the maximum last access date (excludes files unused since n).
    /// 
    /// Corresponds to `/maxlad` option.
    pub max_last_access_date: Option<&'a str>,
    /// Specifies the minimum last access date (excludes files used since n) If n is less than 1900, n specifies the number of days.
    /// Otherwise, n specifies a date in the format YYYYMMDD.
    /// 
    /// Corresponds to `/minlad` option.
    pub min_last_access_date: Option<&'a str>,
}

impl<'a> From<&'a Filter<'a>> for Vec<OsString> {
    fn from(filter: &'a Filter<'a>) -> Self {
        let mut res = Vec::new();
        
        if filter.handle_archive_and_reset {
            res.push(OsString::from("/m"));
        }
        if let Some(attribs) = filter.include_only_files_with_any_of_these_attribs {
            res.push(OsString::from(String::from("/ia:") + Into::<OsString>::into(attribs).to_str().unwrap()));
        }

        if let Some(filter) = filter.file_exclusion_filter.clone() {
            res.append(&mut filter.into());
        }
        if let Some(filter) = filter.directory_exclusion_filter.clone() {
            res.append(&mut filter.into());
        }
        if let Some(filter) = filter.file_and_directory_exclusion_filter {
            res.append(&mut filter.into());
        }

        if let Some(filter) = filter.file_exclusion_filter_exceptions {
            res.append(&mut filter.into());
        }

        if let Some(max_size) = filter.max_size {
            res.push(OsString::from(format!("/max:{}", max_size)));
        }
        if let Some(min_size) = filter.min_size {
            res.push(OsString::from(format!("/min:{}", min_size)));
        }
        
        if let Some(max_age) = filter.max_age {
            res.push(OsString::from(format!("/maxage:{}", max_age)));
        }
        if let Some(min_age) = filter.min_age {
            res.push(OsString::from(format!("/minage:{}", min_age)));
        }

        if let Some(max_lad) = filter.max_last_access_date {
            res.push(OsString::from(format!("/maxlad:{}", max_lad)));
        }
        if let Some(min_lad) = filter.min_last_access_date {
            res.push(OsString::from(format!("/minlad:{}", min_lad)));
        }

        res
    }
}
impl<'a> From<Filter<'a>> for Vec<OsString> {
    fn from(filter: Filter<'a>) -> Self {
        (&filter).into()
    }
}