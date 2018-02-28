use ::{Node, BagRequest, Flag, FlagSet, FlagMap, nodes};
use uri::Uri;

use syn;
use quote;
use failure::Error;

use std::collections::{BTreeMap, VecDeque, HashMap};
use std::any::{TypeId, Any};
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

fn default_val(_: &Any) -> Result<Box<Any>, Error> {
    Ok(Box::new(()) as Box<Any>)
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

    pub fn solve(&self, bag: BagRequest) -> Result<Solution, Error> {
        fn is_done(n: &NodeInstance) -> Option<usize> {
            // TODO
            None
        }

        let start = NodeInstance {
            index: 0,
            parent: 0,
            satisfies: FlagSet::new(),
            meta: Box::new(bag.uri) as _,
            value: Ok(Box::new(default_val) as _),
        };
        let mut work = Working {
            args: bag.args,
            nodes: vec![start],
            new_nodes: Vec::new(),
            queue: WorkingQueue::new(),
            target: bag.target,
            required: bag.required,
        };

        let mut endpoint = None;
        loop {
            // append all new nodes
            work.nodes.append(&mut work.new_nodes);

            // find next node in queue, starting with the highest priority
            let mut node = None;
            for (_, q) in work.queue.iter_mut().rev() {
                node = q.pop_front();
                if node.is_some() { break };
            }
            let node = match node {
                Some(n) => n,
                None => break,
            };

            endpoint = is_done(&work.nodes[node]);
            if endpoint.is_some() { break }         

            // make next search layer
            for t in &self.transforms {
                t.apply(&mut work, node);
            }
        }

        // TODO: Backtrace
        unimplemented!()
    }
}

/// How to create a bag for a specific asset.
pub struct Solution {
    pub bag_expr: quote::Tokens,
}

/// Data used during the resolution of a specific asset.
pub struct Working {
    pub args: FlagMap<String>,
    nodes: Vec<NodeInstance>,
    new_nodes: Vec<NodeInstance>,
    queue: WorkingQueue,
    target: syn::Type,
    required: FlagSet,
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
    pub args: &'work FlagMap<String>,
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
    required: &'work FlagSet,
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
    satisfies: FlagSet,
    meta: Box<Any>,
    value: Result<Box<Fn(&Any) -> Result<Box<Any>, Error>>, Error>,
}

/// Builds an edge between two nodes.
pub struct EdgeBuilder<A: Node, B: Node> {
    priority: i32,
    satis: FlagSet,
    stops: Option<Error>,
    value: Option<Box<Fn(&Any) -> Result<Box<Any>, Error>>>,
    _ph: PhantomData<(A, B)>,
}

impl<A: Node, B: Node> EdgeBuilder<A, B> {
    pub fn new() -> EdgeBuilder<A, B> {
        EdgeBuilder {
            priority: 0,
            satis: FlagSet::new(),
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

    pub fn satisfies_flags(&mut self, flags: &[Flag]) {
        self.satis.extend(flags);
    }

    pub fn satisfies(&mut self, flag: &str) {
        self.satisfies_flag(Flag::new(flag))
    }

    fn build<N: Node>(self, es: &mut Edges<N>, meta: B::Meta) {
        let parent = es.parent;
        let stopped = self.stops.is_some();
        let value = match (self.value, self.stops) {
            (_, Some(e)) => Err(e),
            (Some(f), _) => Ok(f),
            (None, _) => Ok(Box::new(default_val) as _),
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
