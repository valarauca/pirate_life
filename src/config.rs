
use std::{ process::Command, path::{Path,PathBuf}};

use serde::{Serialize,Deserialize};
use toml::from_str;

use super::util::{config_path};
use super::cli::{Cli};

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
        let store_path = self.disk.get_final_location(&cli.output_name);

        // trigger the download
        self.aria2c.run_aria2c(cli.speed.clone(), &cli.url, &download_path);

        // optionally re-encode
        if cli.perform_reencoding {
            self.ffmpeg.build_command(&download_path, &reencode_path)
                .status()?;
            cli.build_flags(&self.propwriter.path, &reencode_path)
                .status()?;
            std::fs::copy(&reencode_path, &store_path)?;
            std::fs::remove_file(&reencode_path)?;
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
        if self.skip_if_local_and_copy(url,output) {
            //short cut to copy over file
            return;
        }

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

    fn skip_if_local_and_copy(&self, url: &str, output: &Path) -> bool {
        // this is a real URL
        if url.trim().starts_with("http") {
            return false;
        }
        let input_path = Path::new(url);
        if !input_path.is_file() {
            return false;
        }

        match std::fs::copy(input_path,output) {
            Ok(_) => true,
            Err(e) => {
                panic!("error:'{:?}' while copying:'{:?}' to:'{:?}'", e, input_path, output);
            }
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

    fn get_final_location<P>(&self, output_name: P) -> PathBuf
    where
        P: AsRef<Path>
    {
        let mut x = self.store_dir.clone();
        x.push(output_name.as_ref().file_name().unwrap());
        x
    }

}


fn to_name_and_extension<'a>(path: &'a Path) -> Option<(&'a str, &'a str)>
{

    let iter_1 = path.file_stem().into_iter().flat_map(|x| x.to_str());
    let iter_2 = path.extension().into_iter().flat_map(|x| x.to_str());

    iter_1.zip(iter_2)
        .next()
}


