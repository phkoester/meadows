// config.rs

//! Configuration-related utilities.

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;
use std::sync::OnceLock;

use thiserror::Error as ThisError;

use crate::collection::Uvec;
use crate::process::ExecType;

// Macros ---------------------------------------------------------------------------------------------------

macro_rules! mod_debug {
  ($is_debug:expr, $($arg:tt)+) => {
    if $is_debug {
      println!("[meadows::config] {}", format_args!($($arg)+));
    }
  };
}

// `ConfigError` --------------------------------------------------------------------------------------------

/// Error type for [`find_config_file`].
#[derive(Debug, PartialEq, ThisError)]
pub enum ConfigError {
  /// Cargo-manifest directory not found.
  #[error("Cargo-manifest directory not found")]
  CargoManifestDirNotFound,
  /// File not found.
  #[error("File not found")]
  FileNotFound,
  /// Invalid file-name pattern.
  #[error("Invalid file-name pattern `{0}`")]
  InvalidFileNamePattern(String),
}

// `FileNameType` -------------------------------------------------------------------------------------------

#[derive(Debug)]
enum FileNameType {
  InvName,
  Name,
  TestName,
  Blank,
}

// Functions ------------------------------------------------------------------------------------------------

/// A general function that helps to find a particular configuration file for a particular executable.
///
/// Whether or not the Rust executable is a standard binary executable, makes a difference. Example and test
/// executables are expected to be run via Cargo, and configuration files for these executables are only
/// looked for at a few well-defined idiomatic locations within the file-system tree of the crate. Binary
/// executables, however, may both be run by Cargo or "out in the wild", their invocation paths may differ
/// from their canonicalized paths, and they must support a wide range of configuration-file locations.
///
/// # Environment Variables
///
/// Oftentimes, configuration files support the usage of environment variables to be expanded when the file
/// is read. If `set_env_vars` is `true`, the function defines a few environment variables containing some
/// information about the executable that is currently running. This is done at most once per process.
///
/// The following variables are set for all executables:
///
/// | Name   | Description
/// | :----- | :-----------
/// | `dir`  | The canonical directory of the executable, as returned by [`dir`](crate::process::dir)
/// | `home` | The current user's home directory, as returned by [`dirs::home_dir`]
/// | `name` | The canonical name of the executable, as returned by [`name`](crate::process::name)
/// | `path` | The canonical path of the executable, as returned by [`path`](crate::process::path)
/// | `pid`  | The process ID (PID) of the executable
///
/// The following variables are set for binary executables only:
///
/// | Name       | Description
/// | :--------  | :----------
/// | `inv_dir`  | The invocation directory of the executable, as returned by [`inv_dir`](crate::process::inv_dir)
/// | `inv_name` | The invocation name of the executable, as returned by [`inv_name`](crate::process::inv_name)
/// | `inv_path` | The invocation path of the executable, as returned by [`inv_path`](crate::process::inv_path)
///
/// The following variables are set for test executables only:
///
/// | Name        | Description
/// | :---------- | :----------
/// | `test_name` | The canonical test name of the executable, as returned by [`test_name`](crate::process::test_name)
///
/// # File Search
///
/// If `is_debug` is `true`, the function outputs additional debug information on `stdout` that helps to
/// trace the file search.
///
/// As an example, let `file_name_pattern` be `"{}config.toml"`. This pattern would produce the following
/// file names used in the search:
///
/// | Name              | Value
/// | :---------------- | :----
/// | `inv_file_name`   | `${inv_name}.config.toml`
/// | `file_name`       | `${name}.config.toml`
/// | `test_file_name`  | `${test_name}.config.toml`
/// | `blank_file_name` | `config.toml`
///
/// `paths` may contain one or more paths separated by the system-dependent path separator. Each path may
/// either point to a particular file or directory. These paths are checked first. If one of the paths points
/// to an existing file, that file is returned immediately. This is true for all executable types.
///
/// During the file search, the following directories are in use:
///
/// | Name               | Description
/// | :----------------- | :----------
/// | `manifest_dir`     | The value of the environment variable `CARGO_MANIFEST_DIR`
/// | `config_local_dir` | A system-dependent local configuration directory, as returned by [`dirs::config_local_dir`]
/// | `config_dir`       | A system-dependent configuration directory, as returned by [`dirs::config_dir`]
///
/// ## Binary Executables
///
/// `exec_type` must be set to [`ExecType::Binary`]. The following paths are checked, in this order:
///
/// Path                                                   | Note
/// :----------------------------------------------------- | :---
/// | `${path}`                                            | `${path}` is each file in `paths`
/// | `${path}/${inv_file_name}`                           | `${path}` is each directory in `paths`
/// | `${path}/${file_name}`                               | `${path}` is each directory in `paths`
/// | `${inv_dir}/${inv_file_name}`                        |
/// | `${dir}/${file_name}`                                |
/// | `${manifest_dir}/src/${file_name}`                   |
/// | `${manifest_dir}/src/bin/${file_name}`               |
/// | `${config_local_dir}/${inv_name}/${inv_file_name}`   |
/// | `${config_local_dir}/${inv_name}/${blank_file_name}` |
/// | `${config_local_dir}/${name}/${file_name}`           |
/// | `${config_local_dir}/${name}/${blank_file_name}`     |
/// | `${config_dir}/${inv_name}/${inv_file_name}`         |
/// | `${config_dir}/${inv_name}/${blank_file_name}`       |
/// | `${config_dir}/${name}/${file_name}`                 |
/// | `${config_dir}/${name}/${blank_file_name}`           |
///
/// ## Example Executables
///
/// `exec_type` must be set to [`ExecType::Example`]. The following paths are checked, in this order:
///
/// | Path                                          | Note
/// | :-------------------------------------------- | :---
/// | `${path}`                                     | `${path}` is each file in `paths`
/// | `${path}/${file_name}`                        | `${path}` is each directory in `paths`
/// | `${manifest_dir}/examples/${file_name}`       |
/// | `${manifest_dir}/examples/${blank_file_name}` |
///
/// ## Doc-Test Executables
///
/// `exec_type` must be set to [`ExecType::DocTest`]. The following paths are checked, in this order:
///
/// | Path                                     | Note
/// | :--------------------------------------- | :---
/// | `${path}`                                | `${path}` is each file in `paths`
/// | `${path}/${test_file_name}`              | `${path}` is each directory in `paths`
/// | `${manifest_dir}/src/${test_file_name}`  |
/// | `${manifest_dir}/src/${blank_file_name}` |
///
/// ## Unit-Test Executables
///
/// `exec_type` must be set to [`ExecType::UnitTest`]. The following paths are checked, in this order:
///
/// | Path                                     | Note
/// | :--------------------------------------- | :---
/// | `${path}`                                | `${path}` is each file in `paths`
/// | `${path}/${test_file_name}`              | `${path}` is each directory in `paths`
/// | `${manifest_dir}/src/${test_file_name}`  |
/// | `${manifest_dir}/src/${blank_file_name}` |
///
/// ## Integration-Test Executables
///
/// `exec_type` must be set to [`ExecType::IntegTest`]. The following paths are checked, in this order:
///
/// | Path                                       | Note
/// | :----------------------------------------- | :---
/// | `${path}`                                  | `${path}` is each file in `paths`
/// | `${path}/${test_file_name}`                | `${path}` is each directory in `paths`
/// | `${manifest_dir}/tests/${test_file_name}`  |
/// | `${manifest_dir}/tests/${blank_file_name}` |
///
/// ## Benchmark-Test Executables
///
/// `exec_type` must be set to [`ExecType::BenchTest`]. The following paths are checked, in this order:
///
/// | Path                                         | Note
/// | :------------------------------------------- | :---
/// | `${path}`                                    | `${path}` is each file in `paths`
/// | `${path}/${test_file_name}`                  | `${path}` is each directory in `paths`
/// | `${manifest_dir}/benches/${test_file_name}`  |
/// | `${manifest_dir}/benches/${blank_file_name}` |
///
/// # Safety
///
/// If `set_env_vars` is `true`, some environment variables are defined using [`env::set_var`], which is not
/// thread-safe. For detailed information, read the "Safety" section of [`env::set_var`].
///
/// Rust executables tend to be multi-threaded. Rust tests are even multi-threaded by default. For
/// multi-threaded executables, using the environment is completely discouraged. However, to some reasonable
/// extent, it should be safe to write to and read from the environment in one thread only. For instance,
/// [`OnceLock`] may help to implement such single-threaded initialization.
///
/// # Errors
///
/// Returns
///
/// - [`ConfigError::CargoManifestDirNotFound`] if `exec_type` denotes an executable to be run by Cargo and
///   the environment variable `CARGO_MANIFEST_DIR` is not set;
/// - [`ConfigError::FileNotFound`] if a configuration file cannot be found;
/// - [`ConfigError::InvalidFileNamePattern`] if `file_name_pattern` does not contain `"{}"`.
///
/// # Examples
///
/// ```
/// use std::env;
/// use std::ffi::OsStr;
/// use std::path::Path;
///
/// use meadows::config;
/// use meadows::process::ExecType;
///
/// let paths = [Path::new("/bin"), Path::new("/usr/bin")];
///
/// let config_file = config::find_config_file(
///   ExecType::Binary,                              // `exec_type`
///   "{}config.toml",                               // `file_name_pattern`
///   true,                                          // `is_debug`
///   Some(&env::join_paths(paths.iter()).unwrap()), // `paths`
///   true,                                          // `set_env_vars`
/// );
/// ```
pub fn find_config_file(
  exec_type: ExecType,
  file_name_pattern: &str,
  is_debug: bool,
  paths: Option<&OsStr>,
  set_env_vars: bool,
) -> Result<PathBuf, ConfigError> {
  use FileNameType::*;

  // Some introductory debug info

  mod_debug!(is_debug, "Checking paths for {} executable", match exec_type {
    ExecType::Binary => "binary",
    ExecType::Example => "example",
    ExecType::DocTest => "doc-test",
    ExecType::UnitTest => "unit-test",
    ExecType::IntegTest => "integration-test",
    ExecType::BenchTest => "benchmark-test",
  });

  mod_debug!(is_debug, "Current directory: {}", {
    match env::current_dir() {
      Ok(dir) => format!("{dir:?}"),
      Err(_) => String::from("-"),
    }
  });

  // If requested, set env vars. This is executed only once

  if set_env_vars {
    self::set_env_vars(exec_type, is_debug);
  }

  // Look for a matching file in `paths` and return early

  if let Some(paths) = paths {
    for path in env::split_paths(paths) {
      if path.is_file() {
        mod_debug!(is_debug, "Checking path {path:?}");
        mod_debug!(is_debug, "Found configuration file {path:?}");
        return Ok(path);
      }
    }
  }

  // Collect tasks

  let mut tasks: Vec<(PathBuf, Vec<FileNameType>)> = vec![];

  // Add tasks for `paths`

  if let Some(paths) = paths {
    for path in env::split_paths(paths) {
      match exec_type {
        ExecType::Binary => tasks.push((path, vec![InvName, Name])),
        ExecType::Example => tasks.push((path, vec![Name])),
        _ => tasks.push((path, vec![TestName])),
      }
    }
  }

  // Add type-specific tasks

  let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from);
  if exec_type != ExecType::Binary && manifest_dir.is_none() {
    return Err(ConfigError::CargoManifestDirNotFound);
  }

  match exec_type {
    ExecType::Binary => {
      tasks.push((crate::process::inv_dir().clone(), vec![InvName]));
      tasks.push((crate::process::dir().clone(), vec![Name]));

      if let Some(ref manifest_dir) = manifest_dir {
        tasks.push((manifest_dir.clone().join("src"), vec![Name]));
        tasks.push((manifest_dir.clone().join("src").join("bin"), vec![Name]));
      }

      let mut config_dirs = Uvec::new();
      if let Some(dir) = dirs::config_local_dir() {
        config_dirs.push(dir);
      }
      if let Some(dir) = dirs::config_dir() {
        config_dirs.push(dir);
      }

      for config_dir in config_dirs {
        tasks.push((config_dir.clone().join(crate::process::inv_name()), vec![InvName, Blank]));
        tasks.push((config_dir.clone().join(crate::process::name()), vec![Name, Blank]));
      }
    }
    ExecType::Example =>
      if let Some(ref manifest_dir) = manifest_dir {
        tasks.push((manifest_dir.clone().join("examples"), vec![Name, Blank]));
      },
    ExecType::DocTest | ExecType::UnitTest =>
      if let Some(ref manifest_dir) = manifest_dir {
        tasks.push((manifest_dir.clone().join("src"), vec![TestName, Blank]));
      },
    ExecType::IntegTest =>
      if let Some(ref manifest_dir) = manifest_dir {
        tasks.push((manifest_dir.clone().join("tests"), vec![TestName, Blank]));
      },
    ExecType::BenchTest =>
      if let Some(ref manifest_dir) = manifest_dir {
        tasks.push((manifest_dir.clone().join("benches"), vec![TestName, Blank]));
      },
  }

  // Collect files from tasks, provide complete debug output

  let inv_name = crate::process::inv_name().to_string_lossy();
  let name = crate::process::name().to_string_lossy();
  let test_name = crate::process::test_name().to_string_lossy();

  let mut files = Uvec::new();
  for task in &tasks {
    for file_name_type in &task.1 {
      let file_name = match file_name_type {
        InvName => replace_in_pattern(file_name_pattern, &inv_name)?,
        Name => replace_in_pattern(file_name_pattern, &name)?,
        TestName => replace_in_pattern(file_name_pattern, &test_name)?,
        Blank => replace_in_pattern(file_name_pattern, "")?,
      };
      let file = task.0.clone().join(file_name);
      if files.push(file.clone()) {
        mod_debug!(is_debug, "Checking path {:?}", file);
      }
    }
  }

  // Eventually, look for file

  for file in files {
    if file.is_file() {
      mod_debug!(is_debug, "Found configuration file {:?}", file);
      return Ok(file.clone());
    }
  }
  Err(ConfigError::FileNotFound)
}

fn replace_in_pattern(pattern: &str, to: &str) -> Result<String, ConfigError> {
  let from = "{}";
  if let Some(index) = pattern.find(from) {
    let ldot = index > 0;
    let ldot_str = if ldot && !to.is_empty() { "." } else { "" };
    let rdot = index < pattern.len() - from.len();
    let rdot_str = if rdot && !to.is_empty() { "." } else { "" };

    let to = format!("{ldot_str}{to}{rdot_str}");
    Ok(pattern.replacen(from, &to, 1))
  } else {
    Err(ConfigError::InvalidFileNamePattern(pattern.to_owned()))
  }
}

/// Defines a few general-purpose environment variables that may be used from within configuration files.
/// This calls [`set_env_vars_impl`] exactly once per process.
fn set_env_vars(exec_type: ExecType, is_debug: bool) {
  static VAL: OnceLock<()> = OnceLock::new();
  VAL.get_or_init(|| set_env_vars_impl(exec_type, is_debug));
}

fn set_env_vars_impl(exec_type: ExecType, is_debug: bool) {
  let set_env_var = |name: &str, val: &OsStr| {
    mod_debug!(is_debug, "Setting `{}` to {:?}", name, val);
    env::set_var(name, val);
  };

  set_env_var("dir", crate::process::dir().as_ref());
  if let Some(dir) = dirs::home_dir() {
    set_env_var("home", dir.as_ref());
  }
  set_env_var("name", crate::process::name());
  set_env_var("path", crate::process::path().as_ref());
  set_env_var("pid", process::id().to_string().as_ref());

  if exec_type == ExecType::Binary {
    set_env_var("inv_dir", crate::process::inv_dir().as_ref());
    set_env_var("inv_name", crate::process::inv_name());
    set_env_var("inv_path", crate::process::inv_path().as_ref());
  }

  if exec_type.is_test() {
    set_env_var("test_name", crate::process::test_name());
  }
}

// Tests ====================================================================================================

#[cfg(test)]
mod tests {
  use super::*;

  // Functions ----------------------------------------------------------------------------------------------

  #[test]
  fn test_replace_in_pattern() -> Result<(), ConfigError> {
    assert_eq!(
      replace_in_pattern("", "name").unwrap_err(),
      ConfigError::InvalidFileNamePattern("".to_owned())
    );
    assert_eq!(
      replace_in_pattern("begend", "name").unwrap_err(),
      ConfigError::InvalidFileNamePattern("begend".to_owned())
    );

    assert_eq!(replace_in_pattern("{}", "name")?, "name");
    assert_eq!(replace_in_pattern("{}", "")?, "");

    assert_eq!(replace_in_pattern("beg{}", "name")?, "beg.name");
    assert_eq!(replace_in_pattern("beg{}", "")?, "beg");

    assert_eq!(replace_in_pattern("{}end", "name")?, "name.end");
    assert_eq!(replace_in_pattern("{}end", "")?, "end");

    assert_eq!(replace_in_pattern("beg{}end", "name")?, "beg.name.end");
    assert_eq!(replace_in_pattern("beg{}end", "")?, "begend");

    Ok(())
  }
}

// EOF
