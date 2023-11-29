//! Performance options

use std::ffi::OsString;

/// Only one Performance choice can be chosen
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PerformanceChoice {
    /// Creates multi-threaded copies with `n` threads. `n` must be an integer between 1 and 128. The default value for `n` is 8.
    /// 
    /// Corresponds to `/mt` option.
    Threads(Option<u8>), // max 128
    /// Specifies the inter-packet gap to free bandwidth on slow lines.
    /// 
    /// Corresponds to `/ipg` option.
    InterPacketGap(usize)
}

impl From<PerformanceChoice> for OsString {
    fn from(pc: PerformanceChoice) -> Self {
        (&pc).into()
    }
}

impl From<&PerformanceChoice> for OsString {
    fn from(pc: &PerformanceChoice) -> Self {
        match pc {
            PerformanceChoice::Threads(threads) => OsString::from(format!("/mt:{}", threads.map(|n| n.clamp(1, 128)).unwrap_or(8))),
            PerformanceChoice::InterPacketGap(gap) => OsString::from(format!("/ipg:{}", gap))
        }
    }
}

/// Enable performance options
#[derive(Debug, Copy, Clone)]
pub struct PerformanceOptions {
    /// Enables multithreading or inter-packet gap
    performance_choice: Option<PerformanceChoice>,

    /// Copies files without using the Windows Copy Offload mechanism.
    /// 
    /// Corresponds to `/nooffload` option.
    dont_offload: bool,
    /// Requests network compression during file transfer, if applicable.
    /// 
    /// Corresponds to `/compress` option.
    request_network_compression: bool,
    /// Don't follow symbolic links and instead create a copy of the link.
    /// 
    /// Corresponds to `/sl` option.
    copy_rather_than_follow_link: bool,
}

impl From<&PerformanceOptions> for Vec<OsString> {
    fn from(po: &PerformanceOptions) -> Self {
        let mut res: Vec<OsString> = Vec::new();

        if let Some(choice) = po.performance_choice {res.push(choice.into())}
        if po.dont_offload {res.push("/nooffload".into())}
        if po.request_network_compression {res.push("/compress".into())}
        if po.copy_rather_than_follow_link {res.push("/sl".into())}

        res
    }
}
impl From<PerformanceOptions> for Vec<OsString> {
    fn from(po: PerformanceOptions) -> Self {
        (&po).into()
    }
}

/// A struct containing retry options
#[derive(Debug, Clone, Copy, Default)]
pub struct RetrySettings {
    /// Specifies the number of retries on failed copies. The default value of n is 1,000,000 (one million retries).
    /// 
    /// Corresponds to `/r` option.
    pub specify_retries_failed_copies: Option<Option<usize>>,
    /// Specifies the wait time between retries, in seconds. The default value of n is 30 (wait time 30 seconds).
    /// 
    /// Corresponds to `/w` option.
    pub specify_wait_between_retries: Option<Option<usize>>,
    /// Saves the values specified in the /r and /w options as default settings in the registry.
    /// 
    /// Corresponds to `/reg` option.
    pub save_specifications: bool,
    /// Specifies that the system waits for share names to be defined (retry error 67).
    /// 
    /// Corresponds to `/tbd` option.
    pub await_share_names_def: bool,
}

impl From<&RetrySettings> for Vec<OsString> {
    fn from(rs: &RetrySettings) -> Self {
        let mut result = Vec::new();

        if let Some(specified) = rs.specify_retries_failed_copies {
            result.push(OsString::from(
                if let Some(n) = specified {
                    format!("/r:{n}")
                } else {
                    "/r:".to_owned()
                }
            ))
        }
        if let Some(specified) = rs.specify_wait_between_retries {
            result.push(OsString::from(
                if let Some(n) = specified {
                    format!("/w:{n}")
                } else {
                    "/w:".to_owned()
                }
            ))
        }
        if rs.save_specifications {
            result.push(OsString::from("/reg"))
        }
        if rs.await_share_names_def {
            result.push(OsString::from("/tbd"))
        }

        result
    }
}
impl From<RetrySettings> for Vec<OsString> {
    fn from(rs: RetrySettings) -> Self {
        (&rs).into()
    }
}
