
use core::fmt;
use std::fmt::{Debug, Formatter};

use petgraph::graph::{DiGraph, NodeIndex};
use rustc_index::bit_set::{GrowableBitSet};
use rustc_middle::ty;
use rustc_span::Span;
use rustc_middle::mir;
use rustc_middle::mir::{visit::Visitor as VisitorMir};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum DependencyGraphNode<'tcx> {
    Local {
        dst: mir::Local
    },
    FunctionCall {
        function: ty::Ty<'tcx>,
    },
    ReturnedToCaller,
    FunctionInput,
    Assembly { spans: Vec<Span> },
}

impl<'tcx> Debug for DependencyGraphNode<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DependencyGraphNode::Local { dst } => write!(f, "{}", dst.index()),
            DependencyGraphNode::FunctionCall { function } => write!(f, "Call: {}", function.to_string()),
            DependencyGraphNode::ReturnedToCaller => write!(f, "Return"),
            DependencyGraphNode::FunctionInput => write!(f, "Input"),
            DependencyGraphNode::Assembly { spans: _ } => write!(f, "Assembly"),
        }
    }
}

pub struct DataFlowTaintTracker<'tcx, 'a> {
    tcx: ty::TyCtxt<'tcx>,
    pub sanitized_locals: GrowableBitSet<mir::Local>,

    pub data_dependency_graph: DiGraph<DependencyGraphNode<'tcx>, Vec<rustc_span::Span>>,
    data_dependency_graph_index: config::Map<DependencyGraphNode<'tcx>, NodeIndex>,
    pub locals_used_in_control_flow: GrowableBitSet<mir::Local>,

    current_body: &'a mir::Body<'tcx>,
}

impl<'tcx, 'a> DataFlowTaintTracker<'tcx, 'a> {
    pub fn new(tcx: ty::TyCtxt<'tcx>, body: &'a mir::Body<'tcx>) -> Self {
        Self {
            tcx,
            sanitized_locals: GrowableBitSet::new_empty(),
            data_dependency_graph: DiGraph::new(),
            data_dependency_graph_index: config::Map::new(),
            locals_used_in_control_flow: GrowableBitSet::new_empty(),
            current_body: body,
        }
    }
}

impl<'tcx, 'a> DataFlowTaintTracker<'tcx, 'a> {
    pub fn get_place_dependency(&self, place: &mir::Place, dependencies: &mut Vec<mir::Local>) {
        dependencies.push(place.local)
    }
    pub fn get_operand_dependency(&self, operand: &mir::Operand<'tcx>, dependencies: &mut Vec<mir::Local>) {
        match operand {
            mir::Operand::Copy(place) | mir::Operand::Move(place) => {
                self.get_place_dependency(place, dependencies)
            }
            mir::Operand::Constant(_) => {},
        }
    }
    pub fn get_rvalue_dependency(&self, rvalue: &mir::Rvalue<'tcx>, dependencies: &mut Vec<mir::Local>) {
        match rvalue {
            mir::Rvalue::Use(op) => {
                self.get_operand_dependency(op, dependencies);
            }
            mir::Rvalue::Repeat(operand, _) => {
                self.get_operand_dependency(operand, dependencies);
            }
            mir::Rvalue::Ref(_region, _kind, src) => {
                self.get_place_dependency(src, dependencies);
            }
            mir::Rvalue::ThreadLocalRef(_defid) => {  }
            mir::Rvalue::AddressOf(_mutability, place) => {
                self.get_place_dependency(place, dependencies);
            },
            mir::Rvalue::Len(place) => {
                self.get_place_dependency(place, dependencies);
            },
            mir::Rvalue::Cast(_kind, operand, _target) => {
                self.get_operand_dependency(operand, dependencies);
            },
            mir::Rvalue::BinaryOp(_operation, box (op_a, op_b)) => {
                self.get_operand_dependency(op_a, dependencies);
                self.get_operand_dependency(op_b, dependencies);
            }
            mir::Rvalue::NullaryOp(_operation, _ty) => {

                },
            mir::Rvalue::UnaryOp(_operation, operand) => {
                self.get_operand_dependency(operand, dependencies);
            },
            mir::Rvalue::Discriminant(place) => {
                self.get_place_dependency(place, dependencies);
            }
            mir::Rvalue::Aggregate(box _kind, fields) => {
                fields.iter().for_each(|op| self.get_operand_dependency(op, dependencies));
            }
            mir::Rvalue::ShallowInitBox(operand, _ty) => {
                self.get_operand_dependency(operand, dependencies);
            },
            mir::Rvalue::CopyForDeref(place) => {
                self.get_place_dependency(place, dependencies);
            },
        };
    }
    pub fn add_data_dependency_edge(&mut self, from: DependencyGraphNode<'tcx>, to: DependencyGraphNode<'tcx>, span: Span) {
        let from_idx = self.data_dependency_graph_index.get(&from);
        let from_idx = if let Some(idx) = from_idx {
            *idx
        } else {
            let idx = self.data_dependency_graph.add_node(from.clone());
            self.data_dependency_graph_index.insert(from, idx);
            idx
        };

        let to_idx = self.data_dependency_graph_index.get(&to);
        let to_idx = if let Some(idx) = to_idx {
            *idx
        } else {
            let idx = self.data_dependency_graph.add_node(to.clone());
            self.data_dependency_graph_index.insert(to, idx);
            idx
        };

        if let Some(edge) = self.data_dependency_graph.find_edge(from_idx, to_idx) {
            self.data_dependency_graph[edge].push(span);
        } else {
            self.data_dependency_graph.add_edge(from_idx, to_idx, vec![span]);
        }
    }
}

impl<'tcx, 'a> VisitorMir<'tcx> for DataFlowTaintTracker<'tcx, 'a> {
    fn visit_body(&mut self,body: &mir::Body<'tcx>,) {
        
        // identify sanitizers
        for (local, local_delc) in body.local_decls.iter_enumerated() {
            if let ty::TyKind::Adt(base, _generics) = local_delc.ty.kind() {
                let base_type_str = self.tcx.def_path_str(base.did());
                if base_type_str == "untrusted_value::UntrustedValue" {
                    self.sanitized_locals.insert(local);
                }
            }
        }

        self.super_body(body);
    }
    fn visit_basic_block_data(&mut self,block:mir::BasicBlock,data: &mir::BasicBlockData<'tcx>,) {
        println!("\n - Basic Block Data: {}", block.index());
        self.super_basic_block_data(block, data);
    }
    fn visit_local_decl(&mut self,local:mir::Local,local_decl: &mir::LocalDecl<'tcx>,) {
        println!("     let {:?} = {:?}", local, local_decl.ty);
        self.super_local_decl(local, local_decl);
    }
    fn visit_statement(&mut self,statement: &mir::Statement<'tcx>,location:mir::Location,) {
        println!("     Statement: {:?} @ {:?}", statement, location);

        let mut dependencies: Vec<mir::Local> = Vec::with_capacity(2);

        match &statement.kind {
            mir::StatementKind::Assign(box (place, rvalue)) => {
                self.get_rvalue_dependency(&rvalue, &mut dependencies);
                for dependency in dependencies.iter() {
                    self.add_data_dependency_edge(DependencyGraphNode::Local {dst: *dependency}, DependencyGraphNode::Local {dst: place.local}, statement.source_info.span);
                }
            },
            mir::StatementKind::Intrinsic(box mir::NonDivergingIntrinsic::CopyNonOverlapping(copy)) => {
                if let mir::Operand::Copy(src) | mir::Operand::Move(src) = &copy.src {
                    if let mir::Operand::Copy(dst) | mir::Operand::Move(dst) = &copy.dst {
                        self.add_data_dependency_edge(DependencyGraphNode::Local {dst: src.local}, DependencyGraphNode::Local {dst: dst.local}, statement.source_info.span);
                    }
                }
            }
            mir::StatementKind::Deinit(_) => {}
            mir::StatementKind::FakeRead(_) => {},
            mir::StatementKind::SetDiscriminant {..} => {},
            mir::StatementKind::StorageLive(_) => {},
            mir::StatementKind::StorageDead(_) => {},
            mir::StatementKind::Retag(_, _) => {},
            mir::StatementKind::PlaceMention(_) => {},
            mir::StatementKind::AscribeUserType(_, _) => {},
            mir::StatementKind::Coverage(_) => {},
            mir::StatementKind::Intrinsic(box mir::NonDivergingIntrinsic::Assume(_)) => {}
            mir::StatementKind::ConstEvalCounter => {},
            mir::StatementKind::Nop => {},
        }
        self.super_statement(statement, location);
    
    }
    fn visit_assign(&mut self, place: &mir::Place<'tcx>, rvalue: &mir::Rvalue<'tcx>, location:mir::Location,) {
        println!("        {:?} <- {:?} @ {:?}", place, rvalue, location);
        self.super_assign(place, rvalue, location);
    }
    fn visit_terminator(&mut self,terminator: &mir::Terminator<'tcx>,location:mir::Location,) {
        println!("     stop {:?}", terminator.kind);
        
        let mut dependencies: Vec<mir::Local> = Vec::with_capacity(2);

        match &terminator.kind {
            mir::TerminatorKind::Goto {..} => {},
            mir::TerminatorKind::SwitchInt {discr, targets: _} => {
                self.get_operand_dependency(&discr, &mut dependencies);
                for dep in dependencies {
                    self.locals_used_in_control_flow.insert(dep);
                }
            },
            mir::TerminatorKind::UnwindResume => {},
            mir::TerminatorKind::UnwindTerminate(_reason) => {},
            mir::TerminatorKind::Return => {
                self.add_data_dependency_edge(
                    DependencyGraphNode::Local { dst: mir::Local::ZERO },
                    DependencyGraphNode::ReturnedToCaller, terminator.source_info.span);
            },
            mir::TerminatorKind::Unreachable => {},
            mir::TerminatorKind::Drop { place: _, target: _, unwind: _, replace: _ } => {},
            mir::TerminatorKind::Call { func, args, destination, target: _, unwind: _, call_source: _, fn_span} => {
                for arg in args {
                    self.get_operand_dependency(&arg.node, &mut dependencies);
                }

                let func_call = DependencyGraphNode::FunctionCall { function: func.ty(self.current_body, self.tcx) };

                for dep in dependencies {
                    self.add_data_dependency_edge(
                        DependencyGraphNode::Local { dst: dep }, 
                        func_call.clone(), fn_span.to_owned());
                }

                self.add_data_dependency_edge(
                    func_call,
                    DependencyGraphNode::Local { dst: destination.local }, fn_span.to_owned());
            },
            mir::TerminatorKind::TailCall {func, args, fn_span} => {
                for arg in args {
                    self.get_operand_dependency(&arg.node, &mut dependencies);
                }

                let func_call = DependencyGraphNode::FunctionCall { function: func.ty(self.current_body, self.tcx) };

                for dep in dependencies {
                    self.add_data_dependency_edge(
                        DependencyGraphNode::Local { dst: dep }, 
                        func_call.clone(), fn_span.to_owned());
                }

                self.add_data_dependency_edge(
                    func_call,
                    DependencyGraphNode::ReturnedToCaller, fn_span.to_owned());
            },
            mir::TerminatorKind::Assert { .. } => {/* maybe todo */},
            mir::TerminatorKind::Yield { value, resume: _, resume_arg, drop: _ } => {
                self.get_operand_dependency(value, &mut dependencies);

                for dep in dependencies {
                    self.add_data_dependency_edge(
                        DependencyGraphNode::Local { dst: dep }, 
                        DependencyGraphNode::ReturnedToCaller, terminator.source_info.span);
                }

                self.add_data_dependency_edge(DependencyGraphNode::FunctionInput, 
                    DependencyGraphNode::Local { dst: resume_arg.local }, terminator.source_info.span);
            },
            mir::TerminatorKind::CoroutineDrop => {},
            mir::TerminatorKind::FalseEdge {..} => {},
            mir::TerminatorKind::FalseUnwind { .. } => {},
            mir::TerminatorKind::InlineAsm {template: _, operands, options: _, line_spans, targets: _, unwind: _} => {
                let mut inputs = dependencies;
                let mut outputs = Vec::new();
                for op in operands {
                    match op {
                        mir::InlineAsmOperand::In { reg: _, value } => {
                            self.get_operand_dependency(value, &mut inputs)
                        }
                        mir::InlineAsmOperand::Out {reg: _, late: _, place} => {
                            if let Some(place) = place {
                                self.get_place_dependency(place, &mut outputs)
                            }
                        }
                        mir::InlineAsmOperand::InOut {
                            reg: _,
                            late: _,
                            in_value,
                            out_place,
                        } => {
                            self.get_operand_dependency(in_value, &mut inputs);
                            if let Some(place) = out_place {
                                self.get_place_dependency(place, &mut outputs)
                            }
                        },
                        mir::InlineAsmOperand::Const {..} => {}
                        mir::InlineAsmOperand::SymFn {..} => {},
                        mir::InlineAsmOperand::Label {..} => {},
                        mir::InlineAsmOperand::SymStatic {..} => {},
                    }
                }

                let asm_block = DependencyGraphNode::Assembly { spans: line_spans.iter().map(Span::to_owned).collect() };

                for dep in inputs {
                    self.add_data_dependency_edge(DependencyGraphNode::Local { dst: dep }, asm_block.clone(), terminator.source_info.span);
                }
                for dep in outputs {
                    self.add_data_dependency_edge(asm_block.clone(), DependencyGraphNode::Local { dst: dep }, terminator.source_info.span);
                }
            },
        }

        self.super_terminator(terminator, location);
    }
    fn visit_operand(&mut self,operand: &mir::Operand<'tcx>,location:mir::Location,) {
        println!("        op {:?}", operand);
        self.super_operand(operand, location);
    }
}
