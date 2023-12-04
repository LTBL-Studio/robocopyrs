//! Robocopyrs is a wrapper for the robocopy command in Windows.
//! 
//! ```ignore
//! use robocopyrs::RobocopyCommand;
//! use robocopyrs::CopyMode;
//! use robocopyrs::FileProperties;
//! use robocopyrs::DirectoryProperties;
//! use std::path::Path;
//! 
//! let command = RobocopyCommand {
//!     source: Path::new("./source"),
//!     destination: Path::new("./destination"),
//!     copy_mode: Some(CopyMode::RESTARTABLE_MODE_BACKUP_MODE_FALLBACK),
//!     structure_and_size_zero_files_only: true,
//!     copy_file_properties: Some(FileProperties::all()),
//!     copy_dir_properties: Some(DirectoryProperties::all()),
//!     ..RobocopyCommand::default()
//! };
//! 
//! command.execute()?;
//! ```

// #![warn(missing_docs)]

pub mod filter;
pub mod properties;
pub mod performance;
pub mod logging;
pub mod exit_codes;

use std::io;
use std::{convert::TryInto, ffi::OsString, ops::Add, path::Path, process::Command};
use std::fmt::Debug;
use thiserror::Error;

use exit_codes::{OkExitCode, ErrExitCode};
use filter::Filter;
use performance::{PerformanceOptions, RetrySettings};
use logging::LoggingOptions;
use properties::{FileProperties, DirectoryProperties};

/// For enums that allow for multiple variants to be 
/// joined into a single variant
pub trait MultipleVariant: Sized + Add<Self> {
    /// get each variant in a multiple-variant
    fn single_variants(&self) -> Vec<Self>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum FileAttributes {
    READ_ONLY,
    ARCHIVE,
    SYSTEM,
    HIDDEN,
    COMPRESSED,
    NOT_CONTENT_INDEXED,
    ENCRYPTED,
    TEMPORARY,
    _MULTIPLE([bool; 8])
}

impl Add for FileAttributes {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_attribs = match self {
            Self::_MULTIPLE(attribs) => attribs,
            attrib => {
                let mut val = 2_u8.pow(attrib.index_of().unwrap() as u32) * 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(attribs) => result_attribs = result_attribs.iter().zip(attribs.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            attrib => result_attribs[attrib.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_attribs)
    }
}

impl From<&FileAttributes> for OsString {
    fn from(fa: &FileAttributes) -> Self {
        let part ;
        OsString::from(match fa {
            FileAttributes::READ_ONLY => "R",
            FileAttributes::ARCHIVE => "A",
            FileAttributes::SYSTEM => "S",
            FileAttributes::HIDDEN => "H",
            FileAttributes::COMPRESSED => "C",
            FileAttributes::NOT_CONTENT_INDEXED => "N",
            FileAttributes::ENCRYPTED => "E",
            FileAttributes::TEMPORARY => "T",
            FileAttributes::_MULTIPLE(props) => {
                part = ['R', 'A', 'S', 'H', 'C', 'N', 'E', 'T'].iter().zip(props.iter()).filter(|(_, exists)| **exists).unzip::<&char, &bool, String, Vec<bool>>().0;
                part.as_str()
            }
        })
    }
}
impl From<FileAttributes> for OsString {
    fn from(fa: FileAttributes) -> Self {
        (&fa).into()
    }
}

impl MultipleVariant for FileAttributes {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(attribs) => {
                Self::VARIANTS.iter().zip(attribs.iter()).filter(|(_, exists)| **exists).unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
    }
}

impl FileAttributes {
    const VARIANTS: [Self; 8] = [
        Self::READ_ONLY,
        Self::ARCHIVE,
        Self::SYSTEM,
        Self::HIDDEN,
        Self::COMPRESSED,
        Self::NOT_CONTENT_INDEXED,
        Self::ENCRYPTED,
        Self::TEMPORARY
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::READ_ONLY => Some(0),
            Self::ARCHIVE => Some(1),
            Self::SYSTEM => Some(2),
            Self::HIDDEN => Some(3),
            Self::COMPRESSED => Some(4),
            Self::NOT_CONTENT_INDEXED => Some(5),
            Self::ENCRYPTED => Some(6),
            Self::TEMPORARY => Some(7),
            _ => None,
        }
    }

    /// Returns a variant containing all available file attributes.
    #[allow(unused)]
    pub fn all() -> Self {
        Self::_MULTIPLE([true; 8])
    }

    /// Returns a variant containing no file attributes.
    #[allow(unused)]
    pub fn none() -> Self {
        Self::_MULTIPLE([false; 8])
    }
}

/// A copy strategy
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum CopyMode {
    /// Copies files in restartable mode.
    /// 
    /// In restartable mode, should a file copy be interrupted, robocopy can pick up where it left off rather than recopying the entire file.
    /// 
    /// Corresponds to `/z` option.
    RESTARTABLE_MODE,
    /// Copies files in backup mode.
    /// 
    /// In backup mode, robocopy overrides file and folder permission settings (ACLs), which might otherwise block access.
    /// 
    /// Corresponds to `/b` option.
    BACKUP_MODE,
    /// Copies files in restartable mode. If file access is denied, switches to backup mode.
    /// 
    /// Corresponds to `/zb` option.
    RESTARTABLE_MODE_BACKUP_MODE_FALLBACK
}

impl From<&CopyMode> for OsString {
    fn from(cm: &CopyMode) -> OsString {
        match cm {
            CopyMode::RESTARTABLE_MODE => OsString::from("/z"),
            CopyMode::BACKUP_MODE => OsString::from("/b"),
            CopyMode::RESTARTABLE_MODE_BACKUP_MODE_FALLBACK => OsString::from("/zb"),
        }
    }
}
impl From<CopyMode> for OsString {
    fn from(cm: CopyMode) -> Self {
        (&cm).into()
    }
}

/// The move strategy
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Move {
    /// Moves files, and deletes them from the source after they're copied.
    /// 
    /// Corresponds to `/mov` option.
    FILES,
    /// Moves files and directories, and deletes them from the source after they're copied.
    /// 
    /// Corresponds to `/move` option.
    FILES_AND_DIRS,
}

impl From<&Move> for OsString {
    fn from(mv: &Move) -> Self {
        match mv {
            Move::FILES => OsString::from("/mov"),
            Move::FILES_AND_DIRS => OsString::from("/move"),
        }
    }
}
impl From<Move> for OsString {
    fn from(mv: Move) -> Self {
        (&mv).into()
    }
}

/// What attributes to add or remove from copied files.
#[derive(Debug, Copy, Clone)]
pub enum PostCopyActions {
    /// Adds the specified attributes to copied files.
    /// 
    /// Corresponds to `/a+` option.
    AddAttribsToFiles(FileAttributes),
    /// Removes the specified attributes from copied files.
    /// 
    /// Corresponds to `/a-` option.
    RmvAttribsFromFiles(FileAttributes),
    _MULTIPLE(FileAttributes, FileAttributes)
}

impl Add for PostCopyActions {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut add_attribs, mut rmv_attribs) = match self {
            Self::_MULTIPLE(add, rmv) => (Some(add), Some(rmv)),
            Self::AddAttribsToFiles(attribs) => (None, Some(attribs)),
            Self::RmvAttribsFromFiles(attribs) => (Some(attribs), None)
        };

        match rhs {
            Self::_MULTIPLE(add, rmv) => {
                if let Some(attribs) = add_attribs {
                    add_attribs = Some(attribs + add);
                }
                if let Some(attribs) = rmv_attribs{
                    rmv_attribs = Some(attribs + rmv);
                }
            },
            Self::AddAttribsToFiles(add) => {
                if let Some(attribs) = add_attribs {
                    add_attribs = Some(attribs + add);
                }
            },
            Self::RmvAttribsFromFiles(rmv) => {
                if let Some(attribs) = rmv_attribs{
                    rmv_attribs = Some(attribs + rmv);
                }
            }
        }

        match (add_attribs, rmv_attribs) {
            (Some(add), Some(rmv)) => Self::_MULTIPLE(add, rmv),
            (None, Some(rmv)) => Self::RmvAttribsFromFiles(rmv),
            (Some(add), None) => Self::AddAttribsToFiles(add),
            (None, None) => panic!("use default rather than PostCopyActions::_MULTIPLE(FileAttributes::none(), FileAttributes::none())")
        }
    }
}

impl From<&PostCopyActions> for Vec<OsString> {
    fn from(pca: &PostCopyActions) -> Self {
        match pca {
            PostCopyActions::AddAttribsToFiles(attribs) => vec![OsString::from(String::from("/a+:") + Into::<OsString>::into(attribs).to_str().unwrap())],
            PostCopyActions::RmvAttribsFromFiles(attribs) => vec![OsString::from(String::from("/a-:") + Into::<OsString>::into(attribs).to_str().unwrap())],
            PostCopyActions::_MULTIPLE(add_attribs, rmv_attribs) => vec![OsString::from(String::from("/a+:") + Into::<OsString>::into(add_attribs).to_str().unwrap()), OsString::from(String::from("/a-:") + Into::<OsString>::into(rmv_attribs).to_str().unwrap())],
        }
    }
}
impl From<PostCopyActions> for Vec<OsString> {
    fn from(pca: PostCopyActions) -> Self {
        (&pca).into()
    }
}

impl MultipleVariant for PostCopyActions {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(add, rmv) => vec![Self::AddAttribsToFiles(*add), Self::RmvAttribsFromFiles(*rmv)],
            variant => vec![*variant]
        }
    }
}

/// Specifies file system options
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum FilesystemOptions {
    /// Creates destination files by using 8.3 character-length FAT file names only.
    /// 
    /// Corresponds to `/fat` option.
    FAT_FILE_NAMES,
    /// Assumes FAT file times (two-second precision).
    /// 
    /// Corresponds to `/fft` option.
    ASSUME_FAT_FILE_TIMES,
    /// Turns off support for paths longer than 256 characters.
    /// 
    /// Corresponds to `/256` option.
    DISABLE_LONG_PATHS,
    _MULTIPLE([bool; 3])
}

impl From<&FilesystemOptions> for Vec<OsString> {
    fn from(fso: &FilesystemOptions) -> Self {
        match fso {
            FilesystemOptions::FAT_FILE_NAMES => vec![OsString::from("/fat")],
            FilesystemOptions::ASSUME_FAT_FILE_TIMES => vec![OsString::from("/fft")],
            FilesystemOptions::DISABLE_LONG_PATHS => vec![OsString::from("/256")],
            FilesystemOptions::_MULTIPLE(options) => ["/fat", "/fft", "/256"].iter().zip(options.iter()).filter(|(_, exists)| **exists).map(|(option, _)| OsString::from(*option)).collect()
        }
    }
}
impl From<FilesystemOptions> for Vec<OsString> {
    fn from(fso: FilesystemOptions) -> Self {
        (&fso).into()
    }
}


/// Robocopy command builder
/// 
#[derive(Debug, Clone)]
pub struct RobocopyCommandBuilder<'a> {
    /// The source's path
    pub source: &'a Path,
    /// The destination's path
    pub destination: &'a Path,
    /// Specifies the file or files to be copied. Wildcard characters are supported.
    pub files: Vec<&'a str>,
    /// Specifies a copy strategy
    pub copy_mode: Option<CopyMode>,
    /// Copies using unbuffered I/O (recommended for large files).
    /// 
    /// Corresponds to `/j` option.
    pub unbuffered: bool,

    /// Copies subdirectories. This option automatically includes empty directories.
    /// 
    /// Corresponds to `/e` option.
    pub empty_dir_copy: bool,
    /// Deletes destination files and directories that no longer exist in the source.
    /// 
    /// Corresponds to `/purge` option.
    pub remove_files_and_dirs_not_in_src: bool,
    /// Copies only the top n levels of the source directory tree.
    /// 
    /// Corresponds to `/lev` option.
    pub only_copy_top_n_levels: Option<usize>,
    /// Creates a directory tree and zero-length files only.
    /// 
    /// Corresponds to `/create` option.
    pub structure_and_size_zero_files_only: bool,
    
    /// Specifies which file properties to copy.
    /// 
    /// Corresponds to `/copy` option.
    pub copy_file_properties: Option<FileProperties>,
    /// Specifies what to copy in directories.
    /// 
    /// Corresponds to `/dcopy` option.
    pub copy_dir_properties: Option<DirectoryProperties>,

    /// Specifies the filter options.
    pub filter: Option<Filter<'a>>,
    
    /// Specifies the file system options.
    pub filesystem_options: Option<FilesystemOptions>,
    /// Specifies the performance options.
    pub performance_options: Option<PerformanceOptions>,
    /// Specifies the retry options.
    pub retry_settings: Option<RetrySettings>,
    
    /// Specifies the logging options.
    pub logging: Option<LoggingOptions<'a>>,
    
    /// Moves file or directories instead of copy.
    pub mv: Option<Move>,
    /// Specifies what attributes to add or remove to copied files
    pub post_copy_actions: Option<PostCopyActions>,

    /// To use this option empty_dir_copy and PostCopyAction::RMV_FILES_AND_DIRS_NOT_IN_SRC must also be in use
    pub overwrite_destination_dir_sec_settings_when_mirror: bool,
    // todo fix secfix and timfix
    // todo job options
}

impl<'a> Default for RobocopyCommandBuilder<'a> {
    fn default() -> Self {
        RobocopyCommandBuilder {
            source: Path::new("."),
            destination: Path::new("."),
            files: Vec::new(),
            copy_mode: None,
            unbuffered: false,
            empty_dir_copy: false,
            remove_files_and_dirs_not_in_src: false,
            only_copy_top_n_levels: None,
            structure_and_size_zero_files_only: false,
            copy_file_properties: None,
            copy_dir_properties: None,
            filter: None,
            filesystem_options: None,
            performance_options: None,
            retry_settings: None,
            logging: None,
            mv: None,
            post_copy_actions: None,
            overwrite_destination_dir_sec_settings_when_mirror: false,
        }
    }
}

impl<'a> RobocopyCommandBuilder<'a> {
    /// Build the command
    pub fn build(&self) -> RobocopyCommand {
        let mut command = Command::new("robocopy");
        
        command
            .arg(self.source)
            .arg(self.destination);

        self.files.iter().for_each(|file| {command.arg(file);});

        if let Some(mode) = &self.copy_mode {
            command.arg(Into::<OsString>::into(mode));
        }
        if self.unbuffered {
            command.arg("/j");
        }
        
        if self.empty_dir_copy && 
                self.remove_files_and_dirs_not_in_src && 
                self.overwrite_destination_dir_sec_settings_when_mirror {
            command.arg("/mir");
            command.arg("/e");
        } else {
            if self.empty_dir_copy {
                command.arg("/e");
            } else {
                command.arg("/s");
            }
            
            if self.remove_files_and_dirs_not_in_src {
                command.arg("/purge");
            }
        }

        if let Some(n) = self.only_copy_top_n_levels {
            command.arg(format!("/lev:{}", n));
        }

        if self.structure_and_size_zero_files_only {
            command.arg("/create");
        }

        if let Some(properties) = self.copy_file_properties {
            command.arg(Into::<OsString>::into(properties));
        }
        if let Some(properties) = self.copy_dir_properties {
            command.arg(Into::<OsString>::into(properties));
        }
        
        if let Some(filter) = &self.filter {
            Into::<Vec<OsString>>::into(filter).into_iter().for_each(|arg| {command.arg(arg);});
        }
        if let Some(options) = &self.filesystem_options {
            Into::<Vec<OsString>>::into(options).into_iter().for_each(|arg| {command.arg(arg);});
        }        
        if let Some(options) = &self.performance_options {
            Into::<Vec<OsString>>::into(options).into_iter().for_each(|arg| {command.arg(arg);});
        }        
        if let Some(settings) = &self.retry_settings {
            Into::<Vec<OsString>>::into(settings).into_iter().for_each(|arg| {command.arg(arg);});
        }

        if let Some(logging) = &self.logging {
            Into::<Vec<OsString>>::into(logging).into_iter().for_each(|arg| {command.arg(arg);});
        }

        if let Some(mv) = &self.mv {
            command.arg(Into::<OsString>::into(mv));
        }
       
        if let Some(actions) = &self.post_copy_actions {
            Into::<Vec<OsString>>::into(actions).into_iter().for_each(|arg| {command.arg(arg);});
        }

        RobocopyCommand { command }        
    }
}

/// A enum on error that can occurs during command execution
#[derive(Error, Debug)]
pub enum Error {
    /// An error occured during copy
    #[error("Error during copy: {0:?}")]
    ExitCode(ErrExitCode),
    /// IO error during command spawning
    #[error("IO error")]
    IoError(#[from] io::Error)
}

impl From<ErrExitCode> for Error {
    fn from(error: ErrExitCode) -> Self {
        Self::ExitCode(error)
    }
}

/// A wrapper around a [Command]
pub struct RobocopyCommand {
    command: Command
}

impl RobocopyCommand {
    /// Executes the command as a child process, waiting for it to finish and returning its status
    pub fn execute(&mut self) -> Result<OkExitCode, Error> {
        let exit_code = self.command.status()?
        .code().expect("Process terminated by signal") as i8;
    
        OkExitCode::try_from(exit_code).map_err(|err| err.into())
    }
}

#[allow(clippy::from_over_into)]
impl Into<Command> for RobocopyCommand {
    /// Converts this robocopy command into a [Command].
    /// Effectively returning the underlying [Command]
    fn into(self) -> Command {
        self.command
    }
}

impl Debug for RobocopyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self.command).replace('\"', ""))
    }
}