//! Logging Options

use std::{ffi::OsString, path::Path};

/// Log file settings
#[derive(Debug, Clone, Copy)]
pub struct LogFileSettings<'a> {
    /// Path to the log file
    pub log: &'a Path,
    /// Writes the log as unicode text. `false` corresponds to `/log` option and `true` to `/unilog` option.
    pub unicode: bool,
    /// Appends output to the existing log file.
    pub append: bool,
}

#[derive(Default, Debug, Clone)]
/// Specify the logging options
pub struct LoggingOptions<'a> {
    /// Specifies that files are to be listed only (and not copied, deleted, or time stamped). Corresponds to `/l` option.
    only_log: bool,
    /// Reports all extra files, not just the ones that are selected. Corresponds to `/x` option.
    report_extra: bool,
    /// Produces verbose output, and shows all skipped files. Corresponds to `/v` option.
    verbose: bool,
    /// Includes source file time stamps in the output. Corresponds to `/ts` option.
    time_stamps: bool,
    /// Includes the full path names of the files in the output. Corresponds to `/fp` option.
    full_path_names: bool,
    /// Prints sizes as bytes. Corresponds to `/bytes` option.
    sizes_bytes: bool,
    /// Specifies that file sizes are not to be logged. Corresponds to `/ns` option.
    dont_log_size: bool,
    /// Specifies that file classes are not to be logged. Corresponds to `/nc` option.
    dont_log_class: bool,
    /// Specifies that file names are not to be logged. Corresponds to `/nfl` option.
    dont_log_file_names: bool,
    /// Specifies that directory names are not to be logged. Corresponds to `/ndl` option.
    dont_log_dir_names: bool,
    /// Specifies that the progress of the copying operation (the number of files or directories copied so far) won't be displayed. Corresponds to `/np` option.
    no_progress_display: bool,
    /// Shows the estimated time of arrival (ETA) of the copied files. Corresponds to `/eta` option.
    show_estimated_time_of_arrival: bool,
    /// Write the status output to a log file.
    log_file: Option<LogFileSettings<'a>>,
    /// Writes the status output to the console window, and to the log file. Corresponds to `/tee` option.
    combination_log: bool,
    /// Specifies that there's no job header. Corresponds to `/njh` option.
    dont_log_header: bool,
    /// Specifies that there's no job summary. Corresponds to `/njs` option.
    dont_log_summary: bool,
    /// Displays the status output as unicode text. Corresponds to `/unicode` option.
    unicode: bool
}

impl<'a> From<&'a LogFileSettings<'a>> for OsString {
    fn from(ls: &'a LogFileSettings<'a>) -> Self {
        OsString::from(
            String::from("/") + 
            if ls.unicode { "uni" } else { "" } + 
            "log" + if ls.append { "+" } else { "" } + 
            ":" + 
            ls.log.to_str().unwrap()
        )
    }
}

impl<'a> From<LogFileSettings<'a>> for OsString {
    fn from(ls: LogFileSettings<'a>) -> Self {
        (&ls).into()
    }
}

impl<'a> From<&'a LoggingOptions<'a>> for Vec<OsString> {
    fn from(lo: &'a LoggingOptions<'a>) -> Self {
        let mut args: Vec<OsString> = Vec::new();
        if lo.only_log { args.push("/l".into())}
        if lo.report_extra { args.push("/x".into()) }
        if lo.verbose { args.push("/v".into()) }
        if lo.time_stamps { args.push("/ts".into()) }
        if lo.full_path_names { args.push("/fp".into()) }
        if lo.sizes_bytes { args.push("/bytes".into()) }
        if lo.dont_log_size { args.push("/ns".into()) }
        if lo.dont_log_class { args.push("/nc".into()) }
        if lo.dont_log_file_names { args.push("/nfl".into()) }
        if lo.dont_log_dir_names { args.push("/ndl".into()) }
        if lo.no_progress_display { args.push("/np".into()) }
        if lo.show_estimated_time_of_arrival { args.push("/eta".into()) }
        if let Some(settings) = lo.log_file { args.push(settings.into()) }
        if lo.combination_log { args.push("/tee".into()) }
        if lo.dont_log_header { args.push("/njh".into()) }
        if lo.dont_log_summary { args.push("/njs".into()) }
        if lo.unicode { args.push("/unicode".into()) }
        args
    }
}