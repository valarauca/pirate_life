
use std::{
    process::Command,
    path::{Path,PathBuf},
    collections::{HashMap},
    borrow::{Cow,ToOwned},
};

use regex::{RegexSet};
use serde::{Serialize,Deserialize};
use toml::from_str;
use win_canonicalize::{canonicalize, move_file};

use super::util::{config_path};
use super::cli::{Cli};
use super::url::UrlLike;

#[derive(Clone,Debug,Serialize,Deserialize,Default)]
pub struct Config {
    aria2c: Aria2c,
    ffmpeg: Ffmpeg,
    propwriter: PropWriter,
    disk: PathManager,
}


impl Config {

    /// reads a configuration from file
    pub fn read_config() -> Result<Self,Box<dyn std::error::Error>> {
        let path = config_path()?;
        let config = std::fs::read_to_string(&path)?;
        Ok(from_str(&config)?)
    }

    pub fn do_it_all(&self, cli: &Cli) -> Result<(),Box<dyn std::error::Error>> {

        let download_path = self.disk.get_download_location(&cli.output_name);
        let reencode_path = self.disk.get_reencode_location(&cli.output_name);
        let store_path = self.disk.get_final_location(&cli.output_name,cli.override_output);

        // trigger the download
        match &cli.url {
            &UrlLike::RemoteHTTP(ref url) => {
                self.aria2c.run_aria2c(cli.speed.clone(), url.as_str(), &download_path);
            },
            &UrlLike::LocalFile(ref path) => {
                move_file(path, download_path.as_path().to_string_lossy().as_ref(), false)?;
            }
        };

        // optionally re-encode
        if cli.perform_reencoding {
            self.ffmpeg.build_command(&download_path, &reencode_path)
                .status()?;
            cli.build_flags(&self.propwriter.path, &reencode_path)
                .status()?;
            move_file(reencode_path.as_path().to_string_lossy().as_ref(), store_path.as_path().to_string_lossy().as_ref(), false)?;
        } else {
            cli.build_flags(&self.propwriter.path, &download_path)
                .status()?;
            std::fs::copy(&download_path, &store_path)?;
        }

        std::fs::remove_file(&download_path)?;

        Ok(())
    }
}

#[derive(Clone,Debug,Serialize,Deserialize,Default)]
pub struct Aria2c {
    path: String,
    default_speed: u64,
    default_args: Vec<String>,
}
impl Aria2c {

    fn run_aria2c(&self, speed: Option<u64>, url: &str, output: &Path) {

        let mut cmd = Command::new(&self.path);
        cmd.current_dir(output.parent().expect("could not recover parent"));
        cmd.args(&self.default_args);
       
        // set maximum speed
        let speed_limit: u64 = if let Some(speed) = speed {
            speed.clone()
        } else {
            self.default_speed.clone()
        };
        cmd.arg(format!("--max-overall-download-limit={}k",speed_limit));

        cmd.arg("-o");
        cmd.arg(output.file_name().expect("could not get file name"));
        cmd.arg(url);
        match cmd.status() {
            Ok(status) => match status.code() {
                Option::Some(0) => return,
                Option::Some(x) => panic!("aria2c returned error {:?}", x),
                Option::None => panic!("aira2c returned known error code"),
            },
            Err(e) => panic!("aira2c failed {:?}", e)
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize,Default)]
pub struct Ffmpeg {
    path: PathBuf,
    default_args: Vec<String>,
}

impl Ffmpeg {
    fn build_command(&self, input_file: &Path, output_file: &Path) -> Command {
        let mut cmd = Command::new(&self.path);
        cmd.arg("-i");
        cmd.arg(input_file);

        // using cmd.args() overwrites all args
        for arg in self.default_args.iter() {
            cmd.arg(arg);
        }
        cmd.arg(output_file);
        cmd
    }
}

#[derive(Clone,Debug,Serialize,Deserialize,Default)]
pub struct PropWriter {
    path: PathBuf
}

#[derive(Clone,Debug,Serialize,Deserialize,Default)]
pub struct PathManager {
    temp_dir: PathBuf,
    store_dir: PathBuf,
    preference: Option<HashMap<PathBuf,String>>,
}

impl PathManager {

    fn get_download_location<P>(&self, output_name: P) -> PathBuf
    where
        P: AsRef<Path>
    {
        let (name, ext) = to_name_and_extension(output_name.as_ref()).unwrap();
        let mut temp_location = self.temp_dir.clone();
        temp_location.push(format!("{}-download.{}", name, ext));
        temp_location
    }


    fn get_reencode_location<P>(&self, output_name: P) -> PathBuf
    where
        P: AsRef<Path>
    {
        let (name, ext) = to_name_and_extension(output_name.as_ref()).unwrap();
        let mut temp_location = self.temp_dir.clone();
        temp_location.push(format!("{}-reencode.{}", name, ext));
        temp_location
    }

    fn get_final_location<P>(&self, output_name: P, fixed_path: bool) -> PathBuf
    where
        P: AsRef<Path>
    {

        // check for fixed path override
        let output_path: String = if fixed_path {
            output_name.as_ref().to_string_lossy().to_string()
        } else {
            // set the default store location
            let mut location: PathBuf = self.store_dir.clone();
            // check if a peferential override is given
            if let Some(preference) = self.preference_location(&output_name) {
                location = preference;
            }
            location.push(&output_name.as_ref().file_name().unwrap());
            location.as_path().to_string_lossy().to_string()
        };
        match canonicalize(&output_path) {
            Ok(x) => {
                let buf = PathBuf::from(x);
                if buf.has_root() {
                    buf
                } else {
                    panic!("cannont canonicalize:'{:?}'", output_name.as_ref());
                }
            }
            Err(e) => panic!("error:'{:?}' cannot canonicalize:'{:?}'", e, output_name.as_ref())
        }
    }

    fn preference_location<P>(&self, output_name: &P) -> Option<PathBuf>
    where
        P: AsRef<Path>
    {
        let data = match &self.preference {
            &Option::None => return None,
            &Option::Some(ref map) => {
                map
            }
        };

        let mut paths = Vec::with_capacity(data.len());
        let mut regexes = Vec::with_capacity(data.len());
        for (k,v) in data.iter() {
            paths.push(PathBuf::from(k));
            regexes.push(v);
        }

        let set = match RegexSet::new(regexes) {
            Ok(set) => set,
            Err(e) => panic!("failed to build regex set. {:?}", e)
        };
        match set.matches(&output_name.as_ref().to_string_lossy()).into_iter().next() {
            Option::None => None,
            Option::Some(x) => Some(PathBuf::from(&paths[x]))
        }
    }
}


fn to_name_and_extension<'a>(path: &'a Path) -> Option<(&'a str, &'a str)> {
    let iter_1 = path.file_stem().into_iter().flat_map(|x| x.to_str());
    let iter_2 = path.extension().into_iter().flat_map(|x| x.to_str());
    iter_1.zip(iter_2)
        .next()
}


