use std::{ process::Command, path::Path };

use structopt::StructOpt;


#[derive(StructOpt,Debug)]
#[structopt(name = "pirate_life", about = "makes pirating and organizing media easier")]
pub struct Cli {

    /*
     * Download parameters
     *
     */

    #[structopt(short = "u", long)]
    pub url: String,
    #[structopt(short = "o", long)]
    pub output_name: String,
    #[structopt(short = "s", long)]
    pub speed: Option<u64>,
    #[structopt(short = "f",long)]
    pub override_output: bool,

    /*
     * Optional Flags
     *
     */
    #[structopt(short = "r", long)]
    pub perform_reencoding: bool,

    /*
     * Media Tags
     *
     */
    #[structopt(short = "t", long)]
    pub tags: Option<Vec<String>>,
    #[structopt(short = "a", long)]
    pub artists: Option<Vec<String>>,
    #[structopt(long)]
    pub producers: Option<Vec<String>>,
    #[structopt(short = "g", long = "genre")]
    pub genre: Option<Vec<String>>,
    #[structopt(short = "w", long)]
    pub writers: Option<Vec<String>>,
    #[structopt(short = "n", long = "name", alias = "title")]
    pub title: Option<String>,
    #[structopt(short = "y", long)]
    pub year: Option<u64>,
    #[structopt(long)]
    pub season: Option<u64>,
    #[structopt(long)]
    pub episode: Option<u64>,
    #[structopt(long)]
    pub series_name: Option<String>,
    #[structopt(long)]
    pub subtitle: Option<String>,
}


impl Cli {


    fn tags_iter<'a>(&'a self) -> (usize,impl Iterator<Item=String> + 'a) {
        (self.tags.as_ref().map_or(0usize, |x| x.len()),
        self.tags.as_ref()
            .into_iter()
            .flat_map(|x| x)
            .map(|x| x.clone()))
    }
    fn artists_iter<'a>(&'a self) -> (usize,impl Iterator<Item=String> + 'a) {
        (self.artists.as_ref().map_or(0usize, |x| x.len()),
        self.artists.as_ref()
            .into_iter()
            .flat_map(|x| x)
            .map(|x| x.clone()))
    }
    fn producers_iter<'a>(&'a self) -> (usize,impl Iterator<Item=String> + 'a) {
        (self.producers.as_ref().map_or(0usize, |x| x.len()),
        self.producers.as_ref()
            .into_iter()
            .flat_map(|x| x)
            .map(|x| x.clone()))
    }
    fn genres_iter<'a>(&'a self) -> (usize,impl Iterator<Item=String> + 'a) {
        (self.genre.as_ref().map_or(0usize, |x| x.len()),
        self.genre.as_ref()
            .into_iter()
            .flat_map(|x| x)
            .map(|x| x.clone()))
    }
    fn writers_iter<'a>(&'a self) -> (usize,impl Iterator<Item=String> + 'a) {
        (self.writers.as_ref().map_or(0usize, |x| x.len()),
        self.writers.as_ref()
            .into_iter()
            .flat_map(|x| x)
            .map(|x| x.clone()))
    }

    fn title_iter<'a>(&'a self) -> (usize, impl Iterator<Item=String> + 'a) {
        (self.title.as_ref().map_or(0, |_| 1),
            self.title.as_ref().into_iter().map(|x| x.clone()))
    }

    fn series_name_iter<'a>(&'a self) -> (usize, impl Iterator<Item=String> + 'a) {
        (self.series_name.as_ref().map_or(0, |_| 1),
            self.series_name.as_ref().into_iter().map(|x| x.clone()))
    }

    fn subtitle_iter<'a>(&'a self) -> (usize, impl Iterator<Item=String> + 'a) {
        (self.subtitle.as_ref().map_or(0, |_| 1),
            self.subtitle.as_ref().into_iter().map(|x| x.clone()))
    }

    fn year_iter<'a>(&'a self) -> (usize, impl Iterator<Item=String> + 'a) {
        (self.year.as_ref().map_or(0, |_| 1),
            self.year.as_ref().into_iter().map(|x| format!("{}", *x)))
    }

    fn season_iter<'a>(&'a self) -> (usize, impl Iterator<Item=String> + 'a) {
        (self.season.as_ref().map_or(0, |_| 1),
            self.season.as_ref().into_iter().map(|x| format!("{}", *x)))
    }

    fn episode_iter<'a>(&'a self) -> (usize, impl Iterator<Item=String> + 'a) {
        (self.episode.as_ref().map_or(0, |_| 1),
            self.episode.as_ref().into_iter().map(|x| format!("{}", *x)))
    }


    /*
     * Abstract way of passing a flag & its contents
     *
     */
    fn add_flags<'a, F,I>(&'a self, flag: &str, getter: F, cmd: &mut Command)
    where 
        I: Iterator<Item=String> + 'a,
        F: Fn(&'a Self) -> (usize,I)
    {
        let (num, tags) = (getter)(self);
        if num == 0 {
            return;
        }
        cmd.arg(flag);
        for (arg,_) in tags.zip(0..num) {
            cmd.arg(arg);
        }
    }


    /// build the various invocations of 
    pub fn build_flags(&self, path: &Path, file: &Path) -> Command {
        let mut cmd = Command::new(path);
        cmd.arg(file);

        self.add_flags("--tags", Self::tags_iter, &mut cmd);
        self.add_flags("--artists", Self::artists_iter, &mut cmd);
        self.add_flags("--producers", Self::producers_iter, &mut cmd);
        self.add_flags("--genre", Self::genres_iter, &mut cmd);
        self.add_flags("--writers", Self::writers_iter, &mut cmd);
        self.add_flags("--title", Self::title_iter, &mut cmd);
        self.add_flags("--series-name", Self::series_name_iter, &mut cmd);
        self.add_flags("--subtitle", Self::subtitle_iter, &mut cmd);
        self.add_flags("--year", Self::year_iter, &mut cmd);
        self.add_flags("--episode", Self::episode_iter, &mut cmd);
        self.add_flags("--season", Self::season_iter, &mut cmd);

        cmd
    }
}
