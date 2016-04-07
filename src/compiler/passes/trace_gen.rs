use ast::ir::*;
use vm::context::VMContext;
use compiler::CompilerPass;

pub struct TraceGen {
    name: &'static str
}

impl TraceGen {
    pub fn new() -> TraceGen {
        TraceGen{name: "Trace Generation"}
    }
}

impl CompilerPass for TraceGen {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn visit_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        // we put the high probability edge into a hot trace, and others into cold paths
        // and traverse cold_path later
        let trace = {
            let mut trace : Vec<MuTag> = vec![];
            let mut work_stack : Vec<MuTag> = vec![];
        
            let entry = func.content.as_ref().unwrap().entry;
            work_stack.push(entry);
            
            while !work_stack.is_empty() {
                let cur = work_stack.pop().unwrap();
                let cur_block = func.content.as_ref().unwrap().get_block(&cur);
                
                trace!("check block {}", cur);
                                
                trace!("add {:?} to trace", cur);
                trace.push(cur);
                
                let hot_edge = {
                    match cur_block.control_flow.get_hottest_succ() {
                        Some(tag) => tag,
                        None => continue
                    }
                };
                
                // push cold paths (that are not in the trace and not in the work_stack) to work_stack
                let mut cold_edges = cur_block.control_flow.succs.clone();
                cold_edges.retain(|x| !x.target.eq(hot_edge) && !trace.contains(&x.target) &&!work_stack.contains(&x.target));
                let mut cold_edge_tags = cold_edges.iter().map(|x| x.target).collect::<Vec<MuTag>>();
                trace!("push cold edges {:?} to work stack", cold_edge_tags);
                work_stack.append(&mut cold_edge_tags);
                
                // if hot edge is not in the trace, push it
                if !trace.contains(&hot_edge) && !work_stack.contains(&hot_edge) {
                    trace!("push hot edge {:?} to work stack", hot_edge); 
                    work_stack.push(hot_edge);
                } else {
                    trace!("hot edge {:?} already in trace, ignore", hot_edge);
                }
                
                trace!("");
            }
            
            trace
        };
        
        func.block_trace = Some(trace);
    }
    
    fn finish_function(&mut self, vm_context: &VMContext, func: &mut MuFunction) {
        debug!("trace for {}", func.fn_name);
        debug!("{:?}", func.block_trace.as_ref().unwrap());
    }
}