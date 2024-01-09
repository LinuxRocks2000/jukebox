// Read a packing list. Caching behavior is enforced for fast recall.
use std::path::Path;
use std::io::BufRead;
use std::io::Seek;
use std::io::Read;


#[derive(Debug, Clone)]
pub struct SongData {
    pub name   : String,
    pub source : String,
    pub album  : Option<String>,
    pub author : Option<String>,
    pub index  : usize
}


pub trait PackListReader {
    fn load(&mut self, index : usize); // load a song into cache. Should do nothing if the requested index is already cached.
    fn cache_ref<'a>(&'a self) -> Option<&'a SongData>;
    fn len(&self) -> usize; 
    fn get<'a>(&'a mut self, index : usize) -> Option<&'a SongData> {
        self.load(index);
        self.cache_ref()
    }
}


pub struct SimplePackList {
    cache  : Option<SongData>,
    file   : seek_bufread::BufReader<std::fs::File>,
    length : usize
}


impl SimplePackList {
    // simple packlist format.
    // line based and human editable.
    pub fn new<T : std::convert::AsRef<Path>>(filename : T) -> Self {
        let mut file = seek_bufread::BufReader::new(std::fs::File::open(filename).unwrap());
        let mut length : usize = 0;
        for line in file.by_ref().lines() {
            if line.unwrap() != "" { // ignore empty lines
                length += 1;
            }
        }
        Self {
            cache  : None,
            file,
            length
        }
    }
}

impl PackListReader for SimplePackList {
    fn load(&mut self, index : usize) {
        if let Some(cache) = &self.cache {
            if cache.index == index {
                return; // do nothing if the cache exists and is already fulfilled with the desired data
            }
        }
        // we've verified the cache is invalid, time to actually pull data
        let mut i : usize = 0;
        if let Err(_) = self.file.rewind() {
            panic!("This packlist processor detects no rewind functionality!");
        }
        for line in self.file.by_ref().lines() {
            if let Ok(line) = line {
                if i == index {
                    let data = line.split(",").collect::<Vec<&str>>();
                    if data.len() >= 2 && data.len() <= 4 {
                        self.cache = Some(SongData {
                            name : data[0].trim().to_string(),
                            source : data[1].trim().to_string(),
                            album : if data.len() >= 3 { Some(data[2].trim().to_string()) } else { None },
                            author : if data.len() == 4 { Some(data[3].trim().to_string()) } else { None },
                            index
                        });
                        return;
                    }
                }
            }
            i += 1;
        }
        self.cache = None; // reading failed, let's not leave possibly poisonous data in the cache
    }

    fn cache_ref<'a>(&'a self) -> Option<&'a SongData> {
        self.cache.as_ref()
    }

    fn len(&self) -> usize {
        self.length
    }
}