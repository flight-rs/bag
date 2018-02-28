use std::sync::Mutex;
use std::fmt::{Debug, Display, Formatter, Error as FmtError};
use std::collections::hash_map::{HashMap, Entry};

lazy_static! {
    static ref FLAGS: Mutex<Flags> = Mutex::new(Flags::new());
}

struct Flags {
    name_to_id: HashMap<String, usize>,
    id_to_name: Vec<String>,
}

impl Flags {
    fn new() -> Flags {
        Flags {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
        }
    }

    fn intern(&mut self, name: String) -> Flag {
        match self.name_to_id.entry(name) {
            Entry::Occupied(e) => Flag { id: *e.get() },
            Entry::Vacant(e) => {
                let name = e.key().clone();
                let id = self.id_to_name.len();
                self.id_to_name.push(name);
                e.insert(id);
                Flag { id }
            },
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag {
    id: usize,
}

impl Flag {
    pub fn new(name: &str) -> Flag {
        FLAGS.lock().unwrap().intern(name.to_owned())
    }

    pub fn name(&self) -> String {
        self.with_name(|name| name.to_owned())
    }

    pub fn with_name<V, F: FnOnce(&str) -> V>(&self, func: F) -> V {
        func(&FLAGS.lock().unwrap().id_to_name[self.id])
    }
}

impl Display for Flag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        self.with_name(|name| write!(f, "{}", name))
    }
}

impl Debug for Flag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        self.with_name(|name| write!(f, "Flag {:?}", name))
    }
}
