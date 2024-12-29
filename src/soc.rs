use crate::error::Error;

use std::process::{Command, Output};

pub type Result<T> = std::result::Result<T, Error>;
pub type Watts = u32;
pub type Bandwidth = u32;
pub type CoreCount = u16;

/// Information about the Silicon chip
pub struct SocInfo {
    /// The CPU brand name string
    pub cpu_brand_name: String,
    /// Number of CPU cores
    pub num_cpu_cores: CoreCount,
    /// Number of GPU cores
    pub num_gpu_cores: CoreCount,
    /// Maximum CPU power in watts (if available)
    pub cpu_max_power: Option<Watts>,
    /// Maximum GPU power in watts (if available)
    pub gpu_max_power: Option<Watts>,
    /// Maximum CPU bandwidth in GB/s (if available)
    pub cpu_max_bw: Option<Bandwidth>,
    /// Maximum GPU bandwidth in GB/s (if available)
    pub gpu_max_bw: Option<Bandwidth>,
    /// Number of efficiency cores
    pub e_core_count: CoreCount,
    /// Number of performance cores
    pub p_core_count: CoreCount,
}

#[derive(Debug, PartialEq)]
enum AppleChip {
    M1,
    M1Pro,
    M1Max,
    M1Ultra,
    M2,
    M2Pro,
    M2Max,
    M2Ultra,
    M3,
    M3Pro,
    M3Max,
    Unknown,
}

struct ChipSpecs {
    cpu_tdp: Watts,
    gpu_tdp: Watts,
    cpu_bw: Bandwidth,
    gpu_bw: Bandwidth,
}

impl AppleChip {
    fn from_brand_string(brand: &str) -> Self {
        match brand {
            s if s.contains("M1 Pro") => AppleChip::M1Pro,
            s if s.contains("M1 Max") => AppleChip::M1Max,
            s if s.contains("M1 Ultra") => AppleChip::M1Ultra,
            s if s.contains("M1") => AppleChip::M1,
            s if s.contains("M2 Pro") => AppleChip::M2Pro,
            s if s.contains("M2 Max") => AppleChip::M2Max,
            s if s.contains("M2 Ultra") => AppleChip::M2Ultra,
            s if s.contains("M2") => AppleChip::M2,
            s if s.contains("M3 Pro") => AppleChip::M3Pro,
            s if s.contains("M3 Max") => AppleChip::M3Max,
            s if s.contains("M3") => AppleChip::M3,
            _ => AppleChip::Unknown,
        }
    }

    fn get_specs(&self) -> ChipSpecs {
        match self {
            AppleChip::M1 => ChipSpecs {
                cpu_tdp: 20,
                gpu_tdp: 20,
                cpu_bw: 70,
                gpu_bw: 70,
            },
            AppleChip::M1Pro => ChipSpecs {
                cpu_tdp: 30,
                gpu_tdp: 30,
                cpu_bw: 200,
                gpu_bw: 200,
            },
            AppleChip::M1Max => ChipSpecs {
                cpu_tdp: 30,
                gpu_tdp: 60,
                cpu_bw: 250,
                gpu_bw: 400,
            },
            AppleChip::M1Ultra => ChipSpecs {
                cpu_tdp: 60,
                gpu_tdp: 120,
                cpu_bw: 500,
                gpu_bw: 800,
            },
            AppleChip::M2 => ChipSpecs {
                cpu_tdp: 25,
                gpu_tdp: 15,
                cpu_bw: 100,
                gpu_bw: 100,
            },
            AppleChip::M2Pro => ChipSpecs {
                cpu_tdp: 30,
                gpu_tdp: 35,
                cpu_bw: 0,
                gpu_bw: 0,
            },
            AppleChip::M2Max => ChipSpecs {
                cpu_tdp: 30,
                gpu_tdp: 40,
                cpu_bw: 0,
                gpu_bw: 0,
            },
            // Add more variants as needed
            _ => ChipSpecs {
                cpu_tdp: 0,
                gpu_tdp: 0,
                cpu_bw: 0,
                gpu_bw: 0,
            },
        }
    }
}

impl SocInfo {
    pub fn new() -> Result<SocInfo> {
        let (cpu_brand_name, num_cpu_cores, e_core_count, p_core_count) = cpu_info(&RealCommand)?;
        let num_gpu_cores = gpu_info(&RealCommand)?;

        let chip = AppleChip::from_brand_string(&cpu_brand_name);
        let specs = chip.get_specs();

        Ok(SocInfo {
            cpu_brand_name,
            num_cpu_cores,
            num_gpu_cores,
            cpu_max_power: Some(specs.cpu_tdp),
            gpu_max_power: Some(specs.gpu_tdp),
            cpu_max_bw: Some(specs.cpu_bw),
            gpu_max_bw: Some(specs.gpu_bw),
            e_core_count: e_core_count,
            p_core_count: p_core_count,
        })
    }
}

// https://github.com/tlkh/asitop/blob/74ebe2cbc23d5b1eec874aebb1b9bacfe0e670cd/asitop/utils.py#L94
const SYSCTL_PATH: &str = "/usr/sbin/sysctl";

fn cpu_info(cmd: &impl SystemCommand) -> Result<(String, u16, u16, u16)> {
    let binary = SYSCTL_PATH;
    let args = &[
        // don't display the variable name
        "-n",
        "machdep.cpu.brand_string",
        "machdep.cpu.core_count",
        "hw.perflevel0.logicalcpu",
        "hw.perflevel1.logicalcpu",
    ];

    let output = cmd.execute(binary, args)?;
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

    let num_performance_cores = match iter.next() {
        Some(s) => s.parse::<u16>()?,
        None => return Err(Error::Parse(buffer.to_string())),
    };

    let num_efficiency_cores = match iter.next() {
        Some(s) => s.parse::<u16>()?,
        None => return Err(Error::Parse(buffer.to_string())),
    };

    Ok((
        cpu_brand_name,
        num_cpu_cores,
        num_performance_cores,
        num_efficiency_cores,
    ))
}

// https://github.com/tlkh/asitop/blob/74ebe2cbc23d5b1eec874aebb1b9bacfe0e670cd/asitop/utils.py#L120
fn gpu_info(cmd: &impl SystemCommand) -> Result<u16> {
    let binary = "/usr/sbin/system_profiler";
    let args = &["-detailLevel", "basic", "SPDisplaysDataType"];

    let output = cmd.execute(binary, args)?;
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

/// Trait for system command execution
pub trait SystemCommand {
    fn execute(&self, binary: &str, args: &[&str]) -> Result<Output>;
}

/// Real command executor
pub struct RealCommand;

impl SystemCommand for RealCommand {
    fn execute(&self, binary: &str, args: &[&str]) -> Result<Output> {
        Ok(Command::new(binary).args(args).output()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    struct MockCommand {
        output: Vec<u8>,
    }

    impl MockCommand {
        fn new(output: &str) -> Self {
            Self {
                output: output.as_bytes().to_vec(),
            }
        }
    }

    impl SystemCommand for MockCommand {
        fn execute(&self, _binary: &str, _args: &[&str]) -> Result<Output> {
            Ok(Output {
                status: std::process::ExitStatus::from_raw(0),
                stdout: self.output.clone(),
                stderr: Vec::new(),
            })
        }
    }

    #[test]
    fn test_gpu_info() {
        let mock_output = r#"Graphics/Displays:
            Apple M2:
              Total Number of Cores: 10"#;
        let cmd = MockCommand::new(mock_output);

        let result = gpu_info(&cmd);
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_cpu_info_success() {
        let mock_output = "Apple M2\n8\n4\n4\n";
        let cmd = MockCommand::new(mock_output);

        let result = cpu_info(&cmd);
        assert!(result.is_ok());
        let (brand, cores, p_cores, e_cores) = result.unwrap();
        assert_eq!(brand, "Apple M2");
        assert_eq!(cores, 8);
        assert_eq!(p_cores, 4);
        assert_eq!(e_cores, 4);
    }

    #[test]
    fn test_cpu_info_missing_core_count() {
        let mock_output = "Apple M2\n";
        let cmd = MockCommand::new(mock_output);

        let result = cpu_info(&cmd);
        assert!(matches!(result, Err(Error::ParseInt { .. })));
    }

    #[test]
    fn test_cpu_info_invalid_core_count() {
        let mock_output = "Apple M2\ninvalid\n";
        let cmd = MockCommand::new(mock_output);

        let result = cpu_info(&cmd);
        assert!(matches!(result, Err(Error::ParseInt { .. })));
    }
}
