use std::ffi::{OsStr, OsString};
use std::iter::FromIterator;
use std::path::PathBuf;
use std::str::FromStr;
use std::vec::Vec;
use structopt::StructOpt;

use crate::errors::CargoPlayError;

#[derive(Debug, Clone, Copy)]
pub(crate) enum RustEdition {
    E2015,
    E2018,
}

impl FromStr for RustEdition {
    type Err = CargoPlayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2018" => Ok(RustEdition::E2018),
            "2015" => Ok(RustEdition::E2015),
            _ => Err(CargoPlayError::InvalidEdition(s.into())),
        }
    }
}

impl Into<String> for RustEdition {
    fn into(self) -> String {
        match self {
            RustEdition::E2015 => "2015".into(),
            RustEdition::E2018 => "2018".into(),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::TrailingVarArg"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::AllowLeadingHyphen"))]
#[structopt(name = "cargo-play", about = "Single file cargo runner.")]
pub(crate) struct Opt {
    #[structopt(short = "d", long = "debug", hidden = true)]
    debug: bool,
    #[structopt(short = "c", long = "clean")]
    pub clean: bool,
    #[structopt(short = "t", long = "toolchain", hidden = true)]
    pub toolchain: Option<String>,
    #[structopt(
        long,
        parse(try_from_os_str = "osstr_to_abspath"),
        raw(validator = "dir_exist")
    )]
    pub cache_dir: Option<PathBuf>,
    #[structopt(
        short = "e",
        long = "edition",
        default_value = "2018",
        raw(possible_values = r#"&["2015", "2018"]"#)
    )]
    pub edition: RustEdition,
    #[structopt(long)]
    pub release: bool,
    #[structopt(long)]
    pub cached: bool,
    #[structopt(
        parse(try_from_os_str = "osstr_to_abspath"),
        raw(required = "true", validator = "file_exist")
    )]
    pub src: PathBuf,
    pub args: Vec<String>,
}

impl Opt {
    /// Generate a string of hash based on the path passed in
    pub fn src_hash(&self) -> String {
        let mut hash = sha1::Sha1::new();

        hash.update(self.src.to_string_lossy().as_bytes());

        base64::encode_config(&hash.digest().bytes()[..], base64::URL_SAFE_NO_PAD)
    }

    pub fn temp_dirname(&self) -> PathBuf {
        format!("cargo-play.{}", self.src_hash()).into()
    }

    fn with_toolchain(mut self, toolchain: Option<String>) -> Self {
        self.toolchain = toolchain;
        self
    }

    pub fn parse(args: Vec<String>) -> Result<Self, ()> {
        if args.len() < 2 {
            Self::clap().print_help().unwrap_or(());
            return Err(());
        }

        let with_cargo = args[1] == "play";
        let mut args = args.into_iter();

        if with_cargo {
            args.next();
        }

        let (toolchains, args): (Vec<String>, Vec<String>) = args.partition(|x| x.starts_with("+"));

        let toolchain = toolchains
            .last()
            .map(|s| String::from_iter(s.chars().skip(1)));

        let mut groups = args.splitn(2, |x| x == "--");
        let mut opt = Opt::from_iter(groups.next().unwrap()).with_toolchain(toolchain);
        if let Some(group) = groups.next() {
            opt.args.extend(group.iter().cloned());
        }
        Ok(opt)
    }
}

/// Convert `std::ffi::OsStr` to an absolute `std::path::PathBuf`
fn osstr_to_abspath(v: &OsStr) -> Result<PathBuf, OsString> {
    if let Ok(r) = PathBuf::from(v).canonicalize() {
        Ok(r)
    } else {
        Err(v.into())
    }
}

/// structopt compataible function to check whether a file exists
fn file_exist(v: String) -> Result<(), String> {
    let p = PathBuf::from(v);
    if !p.is_file() {
        Err(format!("input file does not exist: {:?}", p))
    } else {
        Ok(())
    }
}

/// structopt compataible function to check whether a directory exists
fn dir_exist(v: String) -> Result<(), String> {
    let p = PathBuf::from(v);
    if !p.is_dir() {
        Err(format!("input file does not exist: {:?}", p))
    } else {
        Ok(())
    }
}
