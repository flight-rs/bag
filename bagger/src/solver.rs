use ::Node;
use flag::Flag;

use syn::Type;
use failure::Error;

use std::collections::{BTreeMap, VecDeque, HashSet, HashMap};
use std::any::Any;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::marker::PhantomData;

/// Priority for node exploration.
#[derive(PartialEq, Eq)]
pub struct EdgeOrder {
    /// Most significant property: is the edge an active route?
    pub active: bool,
    /// Second most significant property: how many requirements have been satisfied?
    pub satisfies: u32,
    /// Least significant property: what is the exploration priority?
    pub priority: i32,
}

impl Ord for EdgeOrder {
    fn cmp(&self, other: &EdgeOrder) -> Ordering {
        self.active.cmp(&other.active)
            .then(self.satisfies.cmp(&other.satisfies))
            .then(self.priority.cmp(&other.priority))
    }
}

impl PartialOrd for EdgeOrder {
    fn partial_cmp(&self, other: &EdgeOrder) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Stores and solves asset transform graph.
pub struct Solver {
    pub transforms: Vec<Box<TransformInstance>>,
}

impl Solver {
    pub fn new() -> Solver {
        Solver {
            transforms: Vec::new(),
        }
    }

    /// Add a transformation generator.
    pub fn add_transform(&mut self, t: Box<TransformInstance>) {
        self.transforms.push(t);
    }
}

/// Data used during the resolution of a specific asset.
pub struct Working {
    pub args: HashMap<Flag, String>,
    nodes: Vec<NodeInstance>,
    new_nodes: Vec<NodeInstance>,
    queue: WorkingQueue,
    target: Type,
    required: HashSet<Flag>,
}

type WorkingQueue = BTreeMap<EdgeOrder, VecDeque<usize>>;

/// A dynamically typed transformation generator object.
pub trait TransformInstance: Send + 'static {
    fn apply(&self, working: &mut Working, node: usize);
}

/// Transform instance from a closure.
pub struct FnTransform<N, F> {
    func: F,
    _ph: PhantomData<N>,
}

impl<N, F> FnTransform<N, F> {
    pub fn new(func: F) -> FnTransform<N, F> {
        FnTransform {
            func,
            _ph: PhantomData,
        }
    }
}

impl<N, F> TransformInstance for FnTransform<N, F>
    where N: Node, F: Fn(NodeInput<N>) + Send + 'static
{
    fn apply(&self, working: &mut Working, index: usize) {
        let meta = match working.nodes[index].meta
            .downcast_ref::<N::Meta>()
        {
            Some(m) => m,
            None => return,
        };
        
        let node = NodeInput::<N> {
            args: &working.args,
            meta,
            edges: Edges {
                nodes: &working.nodes,
                new_nodes: &mut working.new_nodes,
                queue: &mut working.queue,
                required: &working.required,
                parent: index,
                _ph: PhantomData,
            },
        };

        (self.func)(node)
    }
}

/// Input node data.
pub struct NodeInput<'work, N: Node> {
    pub args: &'work HashMap<Flag, String>,
    pub meta: &'work N::Meta,
    pub edges: Edges<'work, N>,
}

impl<'work, N: Node> NodeInput<'work, N> {
    pub fn arg(&self, name: &str) -> Option<&str> {
        self.args.get(&Flag::new(name)).map(|v| v.as_str())
    }
}

/// Manage transformations on a node.
pub struct Edges<'work, N: Node> {
    nodes: &'work [NodeInstance],
    new_nodes: &'work mut Vec<NodeInstance>,
    queue: &'work mut WorkingQueue,
    required: &'work HashSet<Flag>,
    parent: usize,
    _ph: PhantomData<N>,
}

impl<'work, N: Node> Edges<'work, N> {
    #[inline(always)]
    pub fn add<E: Node>(&mut self, meta: E::Meta, edge: EdgeBuilder<N, E>) {
        edge.build(self, meta)
    }
}

struct NodeInstance {
    index: usize,
    parent: usize,
    satisfies: HashSet<Flag>,
    meta: Box<Any>,
    value: Result<Box<Fn(&Any) -> Result<Box<Any>, Error>>, Error>,
}

/// Builds an edge between two nodes.
pub struct EdgeBuilder<A: Node, B: Node> {
    priority: i32,
    satis: HashSet<Flag>,
    stops: Option<Error>,
    value: Option<Box<Fn(&Any) -> Result<Box<Any>, Error>>>,
    _ph: PhantomData<(A, B)>,
}

impl<A: Node, B: Node> EdgeBuilder<A, B> {
    pub fn new() -> EdgeBuilder<A, B> {
        EdgeBuilder {
            priority: 0,
            satis: HashSet::new(),
            stops: None,
            value: None,
            _ph: PhantomData,
        }
    }

    pub fn stop(&mut self, err: Error) {
        self.stops = Some(err);
    }

    pub fn value<F>(&mut self, eval: F)
        where F: Fn(&A::Target) -> Result<B::Target, Error> + 'static
    {
        self.value = Some(Box::new(move |input|
            eval(match input.downcast_ref::<A::Target>() {
                Some(r) => r,
                None => bail!("could not cast edge")
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

    fn build<N: Node>(self, es: &mut Edges<N>, meta: B::Meta) {
        fn default_val(_: &Any) -> Result<Box<Any>, Error> {
            Ok(Box::new(()) as Box<Any>)
        }

        let parent = es.parent;
        let stopped = self.stops.is_some();
        let value = match (self.value, self.stops) {
            (_, Some(e)) => Err(e),
            (Some(f), _) => Ok(f),
            (None, _) => Ok(Box::new(default_val) as Box<_>),
        };

        let satisfies = {
            let parent = &es.nodes[parent];
            let mut all = parent.satisfies.clone();
            all.extend(self.satis.intersection(&es.required));
            all
        };
        let satis_count = satisfies.len() as u32;

        let index = es.nodes.len() + es.new_nodes.len();
        let node = NodeInstance {
            index,
            parent,
            satisfies,
            meta: Box::new(meta),
            value,
        };
        es.new_nodes.push(node);

        es.queue
            .entry(EdgeOrder {
                active: !stopped,
                satisfies: satis_count,
                priority: self.priority,
            })
            .or_insert_with(|| VecDeque::new())
            .push_back(index);
    }
}
