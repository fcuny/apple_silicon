use crate::error::Error;

use std::process::Command;

pub type Result<T> = std::result::Result<T, Error>;

/// Information about the Silicon chip
pub struct SocInfo {
    /// The CPU brand name string
    pub cpu_brand_name: String,
    /// Number of CPU cores
    pub num_cpu_cores: u16,
    /// Number of GPU cores
    pub num_gpu_cores: u16,
}

impl SocInfo {
    pub fn new() -> Result<SocInfo> {
        let (cpu_brand_name, num_cpu_cores) = cpu_info()?;

        let num_gpu_cores = gpu_info()?;

        Ok(SocInfo {
            cpu_brand_name,
            num_cpu_cores,
            num_gpu_cores,
        })
    }
}

// https://github.com/tlkh/asitop/blob/74ebe2cbc23d5b1eec874aebb1b9bacfe0e670cd/asitop/utils.py#L94
fn cpu_info() -> Result<(String, u16)> {
    let binary = "/usr/sbin/sysctl";
    let args = &[
        // don't display the variable name
        "-n",
        "machdep.cpu.brand_string",
        "machdep.cpu.core_count",
    ];

    let output = Command::new(binary).args(args).output()?;
    let buffer = String::from_utf8(output.stdout)?;

    let mut iter = buffer.split('\n');
    let cpu_brand_name = match iter.next() {
        Some(s) => s.to_string(),
        None => return Err(Error::Parse(buffer.to_string())),
    };

    let num_cpu_cores = match iter.next() {
        Some(s) => s.parse::<u16>()?,
        None => return Err(Error::Parse(buffer.to_string())),
    };

    Ok((cpu_brand_name, num_cpu_cores))
}

// https://github.com/tlkh/asitop/blob/74ebe2cbc23d5b1eec874aebb1b9bacfe0e670cd/asitop/utils.py#L120
fn gpu_info() -> Result<u16> {
    let binary = "/usr/sbin/system_profiler";
    let args = &["-detailLevel", "basic", "SPDisplaysDataType"];

    let output = Command::new(binary).args(args).output()?;
    let buffer = String::from_utf8(output.stdout)?;

    let num_gpu_cores_line = buffer
        .lines()
        .find(|&line| line.trim_start().starts_with("Total Number of Cores"));

    let num_gpu_cores = match num_gpu_cores_line {
        Some(s) => match s.split(": ").last() {
            Some(s) => s.parse::<u16>()?,
            None => return Err(Error::Parse(buffer.to_string())),
        },
        None => return Err(Error::Parse(buffer.to_string())),
    };

    Ok(num_gpu_cores)
}
