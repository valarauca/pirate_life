
use std::path::{Path,PathBuf};



pub fn config_path() -> Result<PathBuf,std::io::Error> {

    if let Some(override_config) = std::env::var("PIRATE_LIFE_CONFIG").ok().map(|x| PathBuf::from(x)) {
        if is_file(&override_config) {
            return Ok(override_config);
        }
    }

    if let Some(xdg) =  std::env::var("XDG_CONFIG_HOME").ok().map(|x| PathBuf::from(x)) {
        let mut a = xdg.clone();
        a.push("pirate_life");
        a.push("pirate_life.toml");
        if is_file(&a) {
            return Ok(a);
        }


        let mut b = xdg.clone();
        b.push("pirate_life.toml");
        if is_file(&b) {
            return Ok(b);
        }
    }

    if let Some(home) = std::env::var("HOME").ok().map(|x| PathBuf::from(x)) {
        let mut a = home.clone();
        a.push(".config");
        a.push("pirate_life");
        a.push("pirate_life.toml");
        if is_file(&a) {
            return Ok(a);
        }
        
        let mut b = home.clone();
        b.push(".pirate_life.toml");
        if is_file(&b) {
            return Ok(b);
        }
    }

    if let Some(var) = std::env::var("APPDATA").ok().map(|x| PathBuf::from(x)) {
        let mut a = var.clone();
        a.push("pirate_life");
        a.push("pirate_life.toml");
        if is_file(&a) {
            return Ok(a);
        }


        let mut b = var.clone();
        b.push("pirate_life.toml");
        if is_file(&b) {
            return Ok(b);
        }
    }

    if let Some(var) = std::env::var("LOCALAPPDATA").ok().map(|x| PathBuf::from(x)) {
        let mut a = var.clone();
        a.push("pirate_life");
        a.push("pirate_life.toml");
        if is_file(&a) {
            return Ok(a);
        }


        let mut b = var.clone();
        b.push("pirate_life.toml");
        if is_file(&b) {
            return Ok(b);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::Other, "could not find configuration"))
}


fn is_file<P>(arg: P) -> bool
where
    P: AsRef<Path>,
{
    match std::fs::metadata(arg) {
        Ok(m) => m.is_file(),
        Err(_) => false,
    }
}
