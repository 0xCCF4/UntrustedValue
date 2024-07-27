use std::collections::VecDeque;

use graphviz_rust::dot_structures::*;
use graphviz_rust::dot_generator::*;
use petgraph::graph::DiGraph;
use rustc_middle::mir;
use rustc_middle::ty;
use rustc_middle::ty::print;

use crate::analysis::taint_problem::TaintProblem;
use crate::analysis::taint_source::TaintSource;

use super::data_flow::DependencyGraphNode;
use super::data_flow::GraphEdge;

pub struct DataFlowCheckResult<'tsrc> {
    pub taint_problems: Vec<TaintProblem<'tsrc>>,
    #[cfg(feature = "graphs")]
    pub data_dependency_graph: Option<Graph>,
}

pub fn check_data_flow<'tsrc, 'tcx>(
    data_dependency_graph: DiGraph<DependencyGraphNode<'tcx>, GraphEdge>,
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    taint_sources: &'tsrc Vec<TaintSource<'static>>,
    draw_graph: bool,
) -> DataFlowCheckResult<'tsrc> {
    enum Marking {
        TaintSource,
        Tainted,
        TaintSink,
        Untainted,
    }

    let mut markings: config::Map<&DependencyGraphNode<'tcx>, Marking> = config::Map::new();

    let mut sources = VecDeque::new();
    for index in data_dependency_graph.node_indices() {
        let node = &data_dependency_graph[index];
        match node {
            DependencyGraphNode::Local { dst: local, ty } => {
                if let ty::TyKind::Adt(base, _generics) = ty.kind() {
                    let base_type_str = tcx.def_path_str(base.did());
                    if base_type_str == "untrusted_value::UntrustedValue" {
                        markings.insert(node, Marking::TaintSink);
                    }
                }
            }
            DependencyGraphNode::FunctionCall { function, span } => {
                if let ty::TyKind::FnDef(base, generics) = function.kind() {
                    let base_type_str = tcx.def_path_str(base);
                    if base_type_str == "untrusted_value::UntrustedValue::wrap" {
                        markings.insert(node, Marking::TaintSink);
                    } else
                    if base_type_str == "std::convert::From::from" {
                        if let Some(arg) = generics.get(0) {
                            if let Some(ty) = arg.as_type() {
                                if let ty::TyKind::Adt(base, _generics) = ty.kind() {
                                    let base_type_str = tcx.def_path_str(base.did());
                                    if base_type_str == "untrusted_value::UntrustedValue" {
                                        markings.insert(node, Marking::TaintSink);
                                    }
                                }
                            }
                        }
                    } else
                    if base_type_str == "std::env::var" { // TODO
                        markings.insert(node, Marking::TaintSource);
                        sources.push_back(index);
                    }
                }
            }
            _ => {}
        }
    }

    while let Some(node) = sources.pop_front() {
        let neighbours = data_dependency_graph.neighbors_directed(node, petgraph::Direction::Outgoing);
        for neighbour in neighbours {
            let neighbour_node = &data_dependency_graph[neighbour];
            if markings.get(neighbour_node).is_none() {
                if markings.get(neighbour_node).is_none() {
                    markings.insert(neighbour_node, Marking::Tainted);
                    sources.push_back(neighbour);
                }
            }
        }
            
    }


    let mut graph: Vec<Stmt> = Vec::new();

    for index in data_dependency_graph.node_indices() {
        let node = &data_dependency_graph[index];
        let node_name = index.index().to_string();

        let (shape, name) = match node {
            DependencyGraphNode::Local { dst: local, ty } => ("circle", local.index().to_string()),
            _ => ("rectangle", format!("{:?}", node)),
        };

        let node_label = format!("{:?}", name);

        let mut node_attributes = vec![
            Attribute(Id::Plain("shape".to_owned()), Id::Plain(shape.to_owned())),
            Attribute(Id::Plain("label".to_owned()), Id::Plain(node_label)),
        ];

        if let Some(marking) = markings.get(node) {
            match marking {
                Marking::TaintSource => {
                    node_attributes.push(Attribute(Id::Plain("color".to_owned()), Id::Plain("red".to_owned())));
                    node_attributes.push(Attribute(Id::Plain("style".to_owned()), Id::Plain("filled".to_owned())));
                }
                Marking::Tainted => {
                    node_attributes.push(Attribute(Id::Plain("color".to_owned()), Id::Plain("orange".to_owned())));
                    node_attributes.push(Attribute(Id::Plain("style".to_owned()), Id::Plain("filled".to_owned())));
                }
                Marking::TaintSink => {
                    node_attributes.push(Attribute(Id::Plain("color".to_owned()), Id::Plain("green".to_owned())));
                    node_attributes.push(Attribute(Id::Plain("style".to_owned()), Id::Plain("filled".to_owned())));
                }
                Marking::Untainted => {
                    
                }
            }
        }

        graph.push(Node::new(
            NodeId (Id::Plain(node_name), None),
            node_attributes).into());
    }

    for index in data_dependency_graph.edge_indices() {
        let edge = &data_dependency_graph[index];
        if let Some((src, target)) = data_dependency_graph.edge_endpoints(index) {
            let src_name = src.index().to_string();
            let src_node = &data_dependency_graph[src];
            let target_name = target.index().to_string();

            let edge_label = format!("{:?}", edge);
            let color = if matches!(markings.get(src_node), Some(Marking::Tainted | Marking::TaintSource)) {
                "red"
            } else if matches!(markings.get(&data_dependency_graph[target]), Some(Marking::TaintSink)) {
                "green"
            } else {
                "black"
            }.to_owned();

            graph.push(Edge {
                ty: EdgeTy::Pair(
                    Vertex::N(NodeId (Id::Plain(src_name), None)),
                    Vertex::N(NodeId (Id::Plain(target_name), None))
                ),
                attributes: vec![
                    Attribute(Id::Plain("label".to_owned()), Id::Plain(edge_label)),
                    Attribute(Id::Plain("color".to_owned()), Id::Plain(color)),
                ]
            }.into());
        }
    }

    DataFlowCheckResult {
        taint_problems: Vec::new(),
        #[cfg(feature = "graphs")]
        data_dependency_graph: if draw_graph {
            Some(Graph::DiGraph { id: Id::Plain("Dataflowgraph".to_owned()), strict: false, stmts: graph })
        } else {
            None
        },
    }
}
