extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;
extern crate failure;

mod name;

pub use name::Name;
pub use uuid::Uuid;
pub use failure::Error;

use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::fs::File;

/// Information about a specific grab request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrabInfo {
    /// Universal identifier for the grab, used to connect the application and the pack.
    pub uid: Uuid,
    /// The asset path to grab.
    pub path: Name,
    /// A set of required flags.
    #[serde(default)]
    pub require: HashSet<String>,
    /// A set of forbidden flags.
    #[serde(default)]
    pub forbid: HashSet<String>,
    /// Arguments to grabbers.
    #[serde(default)]
    pub args: HashMap<String, String>,
}

impl GrabInfo {
    /// Request the given path to be grabbed with given uuid.
    pub fn new(uid: Uuid, path: Name) -> GrabInfo { GrabInfo {
        uid,
        path,
        require: HashSet::new(),
        forbid: HashSet::new(),
        args: HashMap::new(),
    } }
}

/// A list of requested grabs, to be implemented by `grabber`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requests {
    pub grabs: Vec<GrabInfo>,
}

impl Requests {
    pub fn new() -> Requests { Requests { grabs: Vec::new() } }
}

/// Open the given JSON file as a list of grab requests.
pub fn open_requests<P: AsRef<Path>>(path: P) -> Result<Requests, Error> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

/// Save a list of grab requests to the given file.
pub fn save_requests<P: AsRef<Path>>(path: P, meta: &Requests) -> Result<(), Error> {
    serde_json::to_writer(File::create(path)?, meta)?;
    Ok(())
}
