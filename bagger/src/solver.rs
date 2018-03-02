use ::{Node, BagRequest, Flag, nodes};
use flag::{FlagMap, FlagSet};
use expr::BagExpr;

use proc_macro2::Span;
use syn;
use failure::Error;

use std::collections::{BTreeMap, VecDeque};
use std::any::{Any};
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

fn default_val(_: Box<Any>) -> Result<Box<Any>, Error> {
    Ok(Box::new(()) as Box<Any>)
}

fn get_node(nodes: &[Option<NodeInstance>], index: usize) -> &NodeInstance {
    nodes[index].as_ref().expect("early backtrace")
}

impl PartialOrd for EdgeOrder {
    fn partial_cmp(&self, other: &EdgeOrder) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Stores and solves asset transform graph.
pub struct Solver {
    pub transforms: Vec<Box<TransformInstance>>,
    pub terminals: Vec<Box<Terminal>>,
}

impl Solver {
    pub fn new() -> Solver {
        use nodes::*;

        Solver {
            transforms: Vec::new(),
            terminals: vec![
                Box::new(EndOnProducer) as _,
                Box::new(EndOnGenericProducer) as _,
                Box::new(EndOnTerminate) as _],
        }
    }

    pub fn solve(&self, bag: BagRequest) -> Result<Solution, Error> {
        let start = NodeInstance {
            data: Box::new(nodes::Request(bag.uri)) as _,
            parent: 0,
            satisfies: FlagSet::new(),
            value: Ok(Box::new(default_val) as _),
        };
        let mut work = Working {
            span: bag.span,
            args: bag.args,
            nodes: vec![Some(start)],
            new_nodes: Vec::new(),
            queue: WorkingQueue::new(),
            target: bag.target,
            required: bag.required,
            forbidden: bag.forbidden,
        };
        let mut init_queue = VecDeque::new();
        init_queue.push_back(0);
        work.queue.insert(EdgeOrder {
            active: true,
            satisfies: 0,
            priority: 0,
        }, init_queue);

        let mut endpoint = None;
        'bfs: loop {
            // append all new nodes
            work.nodes.extend(&mut work.new_nodes.drain(..).map(Option::from));

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

            // is solved yet?
            {
                let nodei = get_node(&work.nodes, node);
                for ter in &self.terminals {
                    if ter.terminate(&work, nodei) {
                        endpoint = Some((node, ter));
                        break 'bfs
                    }
                }
            }

            // make next search layer
            for t in &self.transforms {
                t.apply(&mut work, node);
            }
        }

        if let Some((endpoint, terminal)) = endpoint {
            // missing requirements
            if let Some(missing) = work.required
                .difference(&get_node(&work.nodes, endpoint).satisfies)
                .next()
            { bail!("no solution with flag \"{}\"", missing)}

            let val = work.backtrace(endpoint)?;
            Ok(terminal.extract(work, val))
        } else { bail!("no solution (try adding more bagger plugins!)") }
    }
}

/// How to create a bag for a specific asset.
#[derive(Debug)]
pub struct Solution {
    pub bag_expr: BagExpr,
}

/// Data used during the resolution of a specific asset.
pub struct Working {
    pub args: FlagMap<String>,
    nodes: Vec<Option<NodeInstance>>,
    new_nodes: Vec<NodeInstance>,
    queue: WorkingQueue,
    pub target: syn::Type,
    pub required: FlagSet,
    pub forbidden: FlagSet,
    pub span: Span,
}

impl Working {
    fn backtrace(&mut self, from: usize) -> Result<Box<Any>, Error> {
        if from == 0 { return Ok(Box::new(()) as _) }
        if let Some(n) = self.nodes[from].take() {
            (n.value?)(self.backtrace(n.parent)?)
        } else {
            bail!("backtrace loop")
        }
    }
}

type WorkingQueue = BTreeMap<EdgeOrder, VecDeque<usize>>;

/// A dynamically typed transformation generator object.
pub trait TransformInstance: 'static {
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
    where N: Node, F: Fn(NodeInput<N>) + 'static
{
    fn apply(&self, working: &mut Working, index: usize) {
        let data = match get_node(&working.nodes, index).downcast_ref::<N>() {
            Some(m) => m,
            None => return,
        };
        
        let node = NodeInput::<N> {
            span: working.span,
            args: &working.args,
            node: data,
            edges: Edges {
                nodes: &working.nodes,
                new_nodes: &mut working.new_nodes,
                queue: &mut working.queue,
                required: &working.required,
                forbidden: &working.forbidden,
                parent: index,
                _ph: PhantomData,
            },
        };

        (self.func)(node)
    }
}

/// Input node data.
pub struct NodeInput<'work, N: Node> {
    pub span: Span,
    pub args: &'work FlagMap<String>,
    pub node: &'work N,
    pub edges: Edges<'work, N>,
}

impl<'work, N: Node> NodeInput<'work, N> {
    pub fn arg(&self, name: &str) -> Option<&str> {
        self.args.get(&Flag::new(name)).map(|v| v.as_str())
    }
}

/// Manage transformations on a node.
pub struct Edges<'work, N: Node> {
    nodes: &'work [Option<NodeInstance>],
    new_nodes: &'work mut Vec<NodeInstance>,
    queue: &'work mut WorkingQueue,
    required: &'work FlagSet,
    forbidden: &'work FlagSet,
    parent: usize,
    _ph: PhantomData<N>,
}

impl<'work, N: Node> Edges<'work, N> {
    #[inline(always)]
    pub fn add<E: Node>(&mut self, node: E, edge: EdgeBuilder<N, E>) {
        edge.build(node, self)
    }
}

pub struct NodeInstance {
    data: Box<Any>,
    parent: usize,
    pub satisfies: FlagSet,
    value: Result<Box<Fn(Box<Any>) -> Result<Box<Any>, Error>>, Error>,
}

impl NodeInstance {
    pub fn downcast_ref<N: Node>(&self) -> Option<&N> {
        self.data.downcast_ref::<N>()
    }

    pub fn downcast<N: Node>(self) -> Option<Box<N>> {
        self.data.downcast::<N>().ok()
    }

    pub fn is<N: Node>(&self) -> bool {
        self.data.is::<N>()
    }
}

pub trait Terminal {
    fn terminate(&self, &Working, &NodeInstance) -> bool;
    fn extract(&self, Working, Box<Any>) -> Solution;
}

/// Builds an edge between two nodes.
pub struct EdgeBuilder<A: Node, B: Node> {
    priority: i32,
    satis: FlagSet,
    stops: Option<Error>,
    value: Option<Box<Fn(Box<Any>) -> Result<Box<Any>, Error>>>,
    _ph: PhantomData<(A, B)>,
}

impl<A: Node, B: Node> EdgeBuilder<A, B> {
    /// Create a new edge between nodes.
    pub fn new() -> EdgeBuilder<A, B> {
        EdgeBuilder {
            priority: 0,
            satis: FlagSet::new(),
            stops: None,
            value: None,
            _ph: PhantomData,
        }
    }

    /// For some reason, this edge does not exist between the two nodes.
    pub fn stop(&mut self, err: Error) {
        self.stops = Some(err);
    }

    pub fn value<F>(&mut self, eval: F)
        where F: Fn(A::Target) -> Result<B::Target, Error> + 'static
    {
        self.value = Some(Box::new(move |input|
            eval(*match input.downcast::<A::Target>() {
                Ok(r) => r,
                Err(_) => bail!("could not cast edge")
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

    fn build(mut self, n: B, es: &mut Edges<A>) {
        // stop if forbidden flag
        if let Some(&f) = es.forbidden
            .intersection(&self.satis)
            .next()
        { self.stop(format_err!("no solution without flag \"{}\"", f)) }

        // some primitive inputs
        let parent = es.parent;
        let stopped = self.stops.is_some();

        // value function
        let value = match (self.value, self.stops) {
            (_, Some(e)) => Err(e),
            (Some(f), _) => Ok(f),
            (None, _) => Ok(Box::new(default_val) as _),
        };

        // build true set of satisfied flags
        let satisfies = {
            let parent = &get_node(&es.nodes, parent);
            let mut all = parent.satisfies.clone();
            all.extend(self.satis.intersection(&es.required));
            all
        };
        let satis_count = satisfies.len() as u32;

        // create new node instace
        let index = es.nodes.len() + es.new_nodes.len();
        let node = NodeInstance {
            data: Box::new(n),
            parent,
            satisfies,
            value,
        };
        es.new_nodes.push(node);

        // add to explore queue
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
