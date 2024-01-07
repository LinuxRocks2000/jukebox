use std::fs;
use std::error::Error;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::io::BufReader;
use std::fs::File;
use std::collections::VecDeque;
use rodio::{Decoder, OutputStream, Sink};
use gpiod::{Chip, Options, Masked, AsValuesMut};

pub mod fourbyfour;
pub mod rotary;

use fourbyfour::{ FourByFour, FourByFourState };
use rotary::RotaryEncoder;


#[derive(Serialize, Deserialize, Debug)]
struct PackListEntry {
    name : String,
    source : String,
    album : Option<String>
}


fn get_songs_dir() -> Result<PathBuf, Box<dyn Error>> {
    let entries = fs::read_dir(shellexpand::tilde("~/Music").to_mut())?;
    let mut default_exists = false;
    let mut ret : Option<PathBuf> = None;
    for thing in entries {
        let thing = thing?;
        let metadata = thing.metadata()?;
        if metadata.is_dir() {
            let mut packinglist = thing.path();
            packinglist.push("packlist.json");
            if packinglist.exists() {
                let path = thing.path();
                let name = path.file_name();
                if name == Some(std::ffi::OsStr::new("default")) {
                    default_exists = true;
                }
                else {
                    ret = Some(thing.path());
                }
            }
        }
    }
    if let Some(ret) = ret {
        return Ok(ret);
    }
    else if default_exists {
        let mut default_path = PathBuf::new();
        default_path.push("~/Music/default/");
        return Ok(default_path);
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to load song data!")))
}


struct SystemState {
    list             : Vec<PackListEntry>,
    ptr              : usize,
    queue            : VecDeque<usize>, // for playlists, just dump them into queue. Easy!
    keypad           : FourByFour,
    old_keypad_state : FourByFourState,
    numstate         : usize, // we typin' numbers if this is nonzero
    paused           : bool
}


impl SystemState {
    fn update(&mut self) -> bool { // returns whether to continue or not, allowing it to do things like interrupt playback upon user input.
        std::thread::sleep(std::time::Duration::from_millis(100)); // pause 0.1s for the kernel to do other stuff
        // this also prevents static with frequency <100ms from triggering button presses (most static is probably closer to the 10ms range,
        // so it covers quite a lot); because button presses are almost always >100ms, this shouldn't affect user experience.
        let buttons = self.keypad.read_pad();
        let dif = self.old_keypad_state.aint(buttons);
        self.old_keypad_state = buttons;
        for numbutton in 0..10 { // rust ranges are noninclusive at the top, so this is really 0 through 9.
            if dif.released(b'0' + numbutton) {
                self.numstate *= 10;
                self.numstate += numbutton as usize;
                println!("Changed number state to {}!", self.numstate);
            }
        }
        if dif.released(b'*') {
            println!("Starkey");
            if self.numstate > 0 {
                if self.numstate <= self.list.len() {
                    self.queue.push_front(self.numstate - 1);
                    println!("Adding {} to queue", self.numstate);
                }
                else {
                    println!("Somebody what tried ter load a song we don't got!");
                }
                self.numstate = 0;
            }
            else {
                self.paused = !self.paused;
                println!("Set paused state to {}", self.paused);
            }
        }
        if dif.released(b'#') {
            if self.numstate > 0 {
                self.numstate = 0;
            }
            else {
                return false; // just straight up skip
            }
        }
        true
    }

    fn pick(&mut self) -> String { // return the filename of a song
        if let Some(pointer) = self.queue.pop_front() {
            println!("Popped {} off of queue to play NOW (the name is '{}')", pointer, self.list[pointer].name);
            self.ptr = pointer;
        }
        else {
            println!("Picking a song at random!");
            loop {
                let new = rand::random::<usize>() % self.list.len();
                if new != self.ptr {
                    self.ptr = new;
                    break;
                }
            }
        }
        self.list[self.ptr].source.clone()
    }
}


fn main() {
    let mut songdir = get_songs_dir().unwrap(); // TODO: inotify on the ~/Music directory to catch updates (fallback to default when a mounted media source unplugs,
    // for instance) to make THAT work, we'll need to set up udev rules; add a wizard to this application that sets udev rules later.
    let mut packlist_loc = songdir.clone();
    packlist_loc.push("packlist.json");
    let mut packlist_reader = BufReader::new(File::open(packlist_loc).unwrap());
    let mut state = SystemState {
        list             : serde_json::from_reader(packlist_reader).unwrap(),
        ptr              : 0,
        queue            : VecDeque::new(),
        keypad           : FourByFour::new([5, 6, 13, 19], [12, 16, 20, 21]), // GPIO pins based on the diagram at https://simonprickett.dev/raspberry-pi-coding-with-rust-traffic-lights/
        old_keypad_state : FourByFourState::empty(),
        numstate         : 0,
        paused           : true
    };
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    loop {
        let mut this_song_loc = songdir.clone();
        this_song_loc.push(state.pick());
        println!("Readin' from sawng {:?}", this_song_loc);
        let song = BufReader::new(File::open(this_song_loc).unwrap());
        if let Ok(musak) = Decoder::new(song) {
            let player = Sink::try_new(&stream_handle).unwrap();
            player.append(musak);
            while !player.empty() && state.update() {
                if state.paused {
                    player.pause();
                }
                else {
                    player.play();
                }
            }
            let new_song_dir = get_songs_dir().unwrap();
            if new_song_dir != songdir {
                songdir = new_song_dir;
                packlist_loc = songdir.clone();
                packlist_loc.push("packlist.json");
                packlist_reader = BufReader::new(File::open(packlist_loc).unwrap());
                state.list = serde_json::from_reader(packlist_reader).unwrap();
            }
        }
    }
}
