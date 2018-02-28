use uri::Uri;
use syn::{Ident, Lit, Type};
use quote::ToTokens;
use failure::Error;

use std::collections::hash_map::{HashMap, Entry};
use std::collections::{BTreeMap, VecDeque, HashSet};
use std::path::PathBuf;
use std::any::{TypeId, Any};
use std::fmt::{Debug, Display, Formatter, Error as FmtError};
use std::sync::Mutex;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::marker::PhantomData;

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

#[derive(PartialEq, Eq)]
pub struct NodeOrd {
    pub error: bool,
    pub satis: u32,
    pub priority: i32,
}

impl Ord for NodeOrd {
    fn cmp(&self, other: &NodeOrd) -> Ordering {
        (!self.error).cmp(&!other.error)
            .then(self.satis.cmp(&other.satis))
            .then(self.priority.cmp(&other.priority))
    }
}

impl PartialOrd for NodeOrd {
    fn partial_cmp(&self, other: &NodeOrd) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct BagInfo {
    pub uri: Uri,
    pub target: Type,
}

pub struct Solver {
    trans: Vec<TransInst>,
}

impl Solver {
    pub fn register_transform<PE, PM, T>(&mut self, trans: T)
        where T: Transform<PE, PM> + 'static, PE: 'static, PM: 'static
    {
        self.trans.push(TransInst {
            applier: |parent, inst, trans| {
                let mut ctx = NodeContext::<PE, PM> {
                    parent,
                    inst,
                    _ph: PhantomData,
                };
                trans.downcast_ref::<T>()
                    .expect("could not downcast transformer")
                    .apply(&mut ctx);
            },
            trans: Box::new(trans) as Box<Any>,
        })
    }
}

struct Instance {
    nodes: Vec<NodeInst>,
    queues: BTreeMap<NodeOrd, VecDeque<usize>>,
    target: Type,
    required: HashSet<Flag>,
}

struct NodeInst {
    id: usize,
    satis: HashSet<Flag>,
    meta: Box<Any>,
    eval: Result<Box<Fn(&Any) -> Result<Box<Any>, Error>>, Error>,
}

struct TransInst {
    applier: fn(usize, &mut Instance, &Any),
    trans: Box<Any>,
}

impl TransInst {
    pub fn transform(&self, inst: &mut Instance, node: usize) {
        (self.applier)(node, inst, &*self.trans)
    }
}

pub struct NodeContext<'inst, PE, PM> {
    parent: usize,
    inst: &'inst mut Instance,
    _ph: PhantomData<(PE, PM)>,
}

impl<'inst, PE: 'static, PM: 'static> NodeContext<'inst, PE, PM> {
    pub fn push<NE, NM>(&mut self, meta: NM, node: NodeBuilder<PE, NE>)
        where NE: 'static, NM: 'static
    {
        node.build(self.parent, self.inst, meta)
    }

    pub fn create<NE, NM, F>(&mut self, meta: NM, creator: F)
        where F: FnOnce(&mut NodeBuilder<PE, NE>), NE: 'static, NM: 'static
    {
        let mut builder = NodeBuilder::new();
        creator(&mut builder);
        self.push(meta, builder);
    }

    pub fn meta(&self) -> &PM {
        self.inst.nodes[self.parent].meta
            .downcast_ref::<PM>()
            .expect("could not downcast metadata")
    }
}

pub struct NodeBuilder<PE, NE> {
    priority: i32,
    satis: HashSet<Flag>,
    error: Option<Error>,
    eval: Option<Box<Fn(&Any) -> Result<Box<Any>, Error>>>,
    _ph: PhantomData<(PE, NE)>,
}

impl<PE: 'static, NE: 'static> NodeBuilder<PE, NE> {
    pub fn new() -> NodeBuilder<PE, NE> {
        NodeBuilder {
            priority: 0,
            satis: HashSet::new(),
            error: None,
            eval: None,
            _ph: PhantomData,
        }
    }

    pub fn error(&mut self, err: Error) {
        self.error = Some(err);
    }

    pub fn evaluator<F>(&mut self, eval: F)
        where F: Fn(&PE) -> Result<NE, Error> + 'static
    {
        self.eval = Some(Box::new(move |input|
            eval(match input.downcast_ref::<PE>() {
                Some(r) => r,
                None => bail!("could not cast edge (THIS IS A BUG!)")
            }).map(|node| Box::new(node) as Box<Any>)
        ))
    }

    pub fn priority(&mut self, priority: i32) {
        self.priority = priority
    }

    pub fn satisfies_flag(&mut self, flag: Flag) {
        self.satis.insert(flag);
    }

    pub fn satisfies(&mut self, flag: &str) {
        self.satisfies_flag(Flag::new(flag))
    }

    fn build<NM: 'static>(self, parent: usize, inst: &mut Instance, meta: NM) {
        let is_err = self.error.is_some();
        let eval = match (self.eval, self.error) {
            (Some(f), _) => Ok(f),
            (None, Some(e)) => Err(e),
            _ => return,
        };

        let satis = {
            let parent = &inst.nodes[parent];
            let mut all = parent.satis.clone();
            all.extend(self.satis.intersection(&inst.required));
            all
        };
        let satis_count = satis.len() as u32;

        let id = inst.nodes.len();
        let node = NodeInst {
            id,
            satis,
            eval,
            meta: Box::new(meta),
        };
        inst.nodes.push(node);

        inst.queues
            .entry(NodeOrd {
                error: is_err,
                satis: satis_count,
                priority: self.priority,
            })
            .or_insert_with(|| VecDeque::new())
            .push_back(id);
    }


}

pub trait Transform<PE: 'static, PM: 'static> {
    fn apply(&self, ctx: &mut NodeContext<PE, PM>);
}
