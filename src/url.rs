
use std::{ str::FromStr, path::PathBuf};

use url::Url;
use win_canonicalize::canonicalize;

#[derive(Clone,Debug)]
pub enum UrlLike {
    RemoteHTTP(Url),
    LocalFile(String),
}
impl UrlLike {
    fn is_http(&self) -> bool {
        match self {
            &UrlLike::RemoteHTTP(_) => true,
            _ => false
        }
    }

    fn is_file(&self) -> bool {
        match self {
            &UrlLike::LocalFile(_) => true,
            _ => false
        }
    }

    fn new(x: &str) -> Self {
        <Self as FromStr>::from_str(x).unwrap()
    }
}

#[test]
fn ensure_url_like_works() {

    assert!( UrlLike::new("https://google.com").is_http());
    assert!( UrlLike::new("my_video.mp4").is_file());
}


impl FromStr for UrlLike {
    type Err=String;

    fn from_str(s: &str) -> Result<Self,Self::Err> {
        match Url::parse(s) {
            Ok(url) => {
                return match url.scheme() {
                    "http" | "https" => Ok(UrlLike::RemoteHTTP(url)),
                    "file" => {
                        match canonicalize(url.path()) {
                            Ok(x) => Ok(UrlLike::LocalFile(x)),
                            Err(e) => Err(format!("{:?}",e))
                        }
                    },
                    x => {
                        Err(format!("unknown url scheme:'{}' cannot handle url:'{}'",x,s))
                    }
                };
            },
            Err(_) => { }
        };

        match canonicalize(s) {
            Ok(x) => Ok(UrlLike::LocalFile(x)),
            Err(e) => Err(format!("{:?}",e))
        }
    }
}
