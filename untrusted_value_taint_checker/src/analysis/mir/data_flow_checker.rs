use std::collections::VecDeque;

use graphviz_rust::dot_structures::*;
use petgraph::{Direction};
use petgraph::graph::DiGraph;
use petgraph::graph::EdgeReference;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use rustc_middle::ty;

use crate::analysis::taint_problem::{TaintProblem, TaintProblemType, TaintUsageType};
use crate::analysis::taint_source::TaintSource;
use crate::IRSpan;

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
    enum Marking<'tsrc> {
        TaintSource(&'tsrc TaintSource<'static>),
        Tainted,
        TaintSink,
    }

    let mut markings: config::Map<&DependencyGraphNode<'tcx>, Marking<'tsrc>> = config::Map::new();

    let mut sources = Vec::new();
    for index in data_dependency_graph.node_indices() {
        let node = &data_dependency_graph[index];
        match node {
            DependencyGraphNode::Local { dst: _, ty } => {
                if let ty::TyKind::Adt(base, _generics) = ty.kind() {
                    let base_type_str = tcx.def_path_str(base.did());
                    if base_type_str == "untrusted_value::UntrustedValue" {
                        markings.insert(node, Marking::TaintSink);
                    }
                }
            }
            DependencyGraphNode::FunctionCall { function, span: _ } => {
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
                    if base_type_str == "std::convert::Into::into" {
                        if let Some(arg) = generics.get(1) {
                            if let Some(ty) = arg.as_type() {
                                if let ty::TyKind::Adt(base, _generics) = ty.kind() {
                                    let base_type_str = tcx.def_path_str(base.did());
                                    if base_type_str == "untrusted_value::UntrustedValue" {
                                        markings.insert(node, Marking::TaintSink);
                                    }
                                }
                            }
                        }
                    }
                    for taint_source in taint_sources {
                        if taint_source.functions.contains(&base_type_str.as_str()) {
                            markings.insert(node, Marking::TaintSource(&taint_source));
                            sources.push(index);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut queue = VecDeque::from(sources.clone());
    while let Some(node) = queue.pop_front() {
        let neighbours = data_dependency_graph.neighbors_directed(node, petgraph::Direction::Outgoing);
        for neighbour in neighbours {
            let neighbour_node = &data_dependency_graph[neighbour];
            if markings.get(neighbour_node).is_none() {
                if markings.get(neighbour_node).is_none() {
                    markings.insert(neighbour_node, Marking::Tainted);
                    queue.push_back(neighbour);
                }
            }
        }
    }
    drop(queue);

    let mut taint_problems = Vec::new();
    'source_test: for source in sources {
        let source_node = &data_dependency_graph[source];
        let mut queue = VecDeque::from(vec![source]);
        let mut visited = Vec::new();
        let taint_source = if let Marking::TaintSource(taint_source) = markings.get(source_node).unwrap() {
            *taint_source
        } else {
            unreachable!("taint source is always found");
        };
        let source_func_sig = if let DependencyGraphNode::FunctionCall { function, span: _} = source_node {
            if let ty::TyKind::FnDef(base, _generics) = function.kind() {
                let base_type_str = tcx.def_path_str(base);
                base_type_str
            } else {
                unreachable!("taint source is always function");
            }
        } else {
            unreachable!("taint source is always function");
        };

        let taint_source_span;
        let mut edges_traversed = Vec::new();

        // initial edge is different
        if let Some(source) = queue.pop_front() {
            visited.push(source);
            let edge: Vec<EdgeReference<'_, GraphEdge>> = data_dependency_graph.edges_directed(source, Direction::Outgoing).collect();
            if edge.len() > 1 {
                taint_problems.push(TaintProblem {
                    taint_source,
                    source_func_sig,
                    source_span: IRSpan::new(edge.iter().nth(0).unwrap().weight().instances.iter().nth(0).unwrap().span, tcx),
                    problem_type: TaintProblemType::Duplicated {
                        statement_chain: Vec::new(),
                        targets: edge.iter().map(|e| e.weight().instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                    }
                });
                continue 'source_test;
            } else if edge.len() == 1 {
                let edge = edge.iter().nth(0).unwrap();
                if edge.weight().is_move_only() {
                    // ok
                    queue.push_back(edge.target());
                    taint_source_span = edge.weight().instances.iter().nth(0).unwrap().span;
                } else {
                    taint_problems.push(TaintProblem {
                        taint_source,
                        source_func_sig,
                        source_span: IRSpan::new(edge.weight().instances.iter().nth(0).unwrap().span, tcx),
                        problem_type: TaintProblemType::Used {
                            statement_chain: Vec::new(),
                            used_in: IRSpan::new(edge.weight().instances.iter().nth(0).unwrap().span, tcx),
                            usage_type: TaintUsageType::Copied,
                        }
                    });
                    continue 'source_test;
                }
            } else {
                continue 'source_test;
            }

            while let Some(node) = queue.pop_front() {
                let edge = {
                    let last_node = visited.last().unwrap_or(&source);
                    let edge = data_dependency_graph.edges_connecting(*last_node, node).nth(0).unwrap();
                    edges_traversed.push(edge.weight());
                    edge.weight()
                };

                if visited.contains(&node) {
                    taint_problems.push(TaintProblem {
                        taint_source,
                        source_func_sig,
                        source_span: IRSpan::new(taint_source_span, tcx),
                        problem_type: TaintProblemType::DataDependencyLoop {
                            statement_chain: edges_traversed.iter().take(edges_traversed.len() - 1).map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                            loop_closure: IRSpan::new(edge.instances.iter().nth(0).unwrap().span, tcx),
                        }
                    });
                    continue 'source_test;
                }
                visited.push(node);
                let neighbours: Vec<NodeIndex> = data_dependency_graph.neighbors_directed(node, petgraph::Direction::Outgoing).collect();

                if neighbours.len() > 1 {
                    let target_edges = neighbours.iter().map(|n| data_dependency_graph.edges_connecting(node, *n).nth(0).unwrap().weight());
                    taint_problems.push(TaintProblem {
                        taint_source,
                        source_func_sig,
                        source_span: IRSpan::new(taint_source_span, tcx),
                        problem_type: TaintProblemType::Duplicated {
                            statement_chain: edges_traversed.iter().map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                            targets: target_edges.map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                        }
                    });
                    continue 'source_test;
                } else if neighbours.len() == 1 {
                    let neighbour_node = &data_dependency_graph[neighbours[0]];
                    let edge = data_dependency_graph.edges_connecting(node, neighbours[0]).nth(0).unwrap().weight();
                    match neighbour_node {
                        DependencyGraphNode::Local { .. } => {
                            if let Some(taint) = markings.get(&data_dependency_graph[neighbours[0]]) {
                                match taint {
                                    Marking::TaintSink => {},
                                    Marking::TaintSource(_) | Marking::Tainted => {
                                        if edge.is_move_only() {
                                            // propagate analysis
                                            queue.push_back(neighbours[0]);
                                        } else {
                                            taint_problems.push(TaintProblem {
                                                taint_source,
                                                source_func_sig: source_func_sig.clone(),
                                                source_span: IRSpan::new(taint_source_span, tcx),
                                                problem_type: TaintProblemType::Used {
                                                    statement_chain: edges_traversed.iter().map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                                                    used_in: IRSpan::new(edge.instances.iter().nth(0).unwrap().span, tcx),
                                                    usage_type: TaintUsageType::Copied,
                                                }
                                            });
                                            continue 'source_test;
                                        }
                                    }
                                }
                            } else {
                                unreachable!("neighbour is always marked");
                            }
                        }
                        DependencyGraphNode::FunctionCall {function: _, span} => {
                            if let Some(taint) = markings.get(&data_dependency_graph[neighbours[0]]) {
                                match taint {
                                    Marking::TaintSink => {},
                                    Marking::TaintSource(_) | Marking::Tainted => {
                                        taint_problems.push(TaintProblem {
                                            taint_source,
                                            source_func_sig: source_func_sig.clone(),
                                            source_span: IRSpan::new(taint_source_span, tcx),
                                            problem_type: TaintProblemType::Used {
                                                statement_chain: edges_traversed.iter().map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                                                used_in: IRSpan::new(edge.instances.iter().nth(0).unwrap().span, tcx),
                                                usage_type: TaintUsageType::FunctionCall(IRSpan::new(*span, tcx)),
                                            }
                                        });
                                    }
                                }
                            } else {
                                unreachable!("neighbour is always marked");
                            }
                        }
                        DependencyGraphNode::ReturnedToCaller => {
                            taint_problems.push(TaintProblem {
                                taint_source,
                                source_func_sig: source_func_sig.clone(),
                                source_span: IRSpan::new(taint_source_span, tcx),
                                problem_type: TaintProblemType::Used {
                                    statement_chain: edges_traversed.iter().map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                                    used_in: IRSpan::new(edge.instances.iter().nth(0).unwrap().span, tcx),
                                    usage_type: TaintUsageType::ReturnedToCaller,
                                }
                            });
                        }
                        DependencyGraphNode::Assembly { .. } => {
                            taint_problems.push(TaintProblem {
                                taint_source,
                                source_func_sig: source_func_sig.clone(),
                                source_span: IRSpan::new(taint_source_span, tcx),
                                problem_type: TaintProblemType::Used {
                                    statement_chain: edges_traversed.iter().map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                                    used_in: IRSpan::new(edge.instances.iter().nth(0).unwrap().span, tcx),
                                    usage_type: TaintUsageType::Assembly,
                                }
                            });
                        }
                        DependencyGraphNode::ControlFlow { .. } => {
                            taint_problems.push(TaintProblem {
                                taint_source,
                                source_func_sig: source_func_sig.clone(),
                                source_span: IRSpan::new(taint_source_span, tcx),
                                problem_type: TaintProblemType::Used {
                                    statement_chain: edges_traversed.iter().map(|e| e.instances.iter().nth(0).unwrap().span).map(|v| IRSpan::new(v, tcx)).collect(),
                                    used_in: IRSpan::new(edge.instances.iter().nth(0).unwrap().span,tcx),
                                    usage_type: TaintUsageType::ControlFlow,
                                }
                            });
                        }
                        DependencyGraphNode::FunctionInput => {
                            unreachable!("Function input is not a child");
                        }
                    }
                }
            }
        }
    }

    let mut graph: Vec<Stmt> = Vec::new();

    for index in data_dependency_graph.node_indices() {
        let node = &data_dependency_graph[index];
        let node_name = index.index().to_string();

        let (shape, name) = match node {
            DependencyGraphNode::Local { dst: local, ty: _ } => ("circle", local.index().to_string()),
            _ => ("rectangle", format!("{:?}", node)),
        };

        let node_label = format!("{:?}", name);

        let mut node_attributes = vec![
            Attribute(Id::Plain("shape".to_owned()), Id::Plain(shape.to_owned())),
            Attribute(Id::Plain("label".to_owned()), Id::Plain(node_label)),
        ];

        if let Some(marking) = markings.get(node) {
            match marking {
                Marking::TaintSource(_) => {
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
            let color = if matches!(markings.get(src_node), Some(Marking::Tainted | Marking::TaintSource(_))) {
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
        taint_problems,
        #[cfg(feature = "graphs")]
        data_dependency_graph: if draw_graph {
            Some(Graph::DiGraph { id: Id::Plain("Dataflowgraph".to_owned()), strict: false, stmts: graph })
        } else {
            None
        },
    }
}
