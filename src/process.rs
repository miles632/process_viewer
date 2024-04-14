#![allow(unused)]
use std::collections::HashMap;
use std::vec::Vec;
use std::cmp::{Ord,Ordering};

use sysinfo::{Process, ProcessStatus, Uid};

#[derive(Clone)]
struct ZProcess {
    // username: String,
    pid: u32,
    uid: u32,
    cpu: f32,
    memory: u64,
    command: Vec<String>,
    virt_mem: u64,
    starttime: u64,
    status: ProcessStatus,
}

impl ZProcess {
    pub fn new(sys_proc: &Process) -> Self {
        ZProcess {
            pid: sys_proc.pid().as_u32(),
            uid: *(sys_proc.user_id().unwrap()).clone(), // TODO: this is weird
            cpu: sys_proc.cpu_usage(),
            memory: sys_proc.memory(),
            command: sys_proc.cmd().to_vec(),
            virt_mem: sys_proc.virtual_memory(),
            starttime: sys_proc.start_time(),
            status: sys_proc.status(),
        }
    }

    // use for comparison within the Subtrees list
    pub fn from_pid_as_zeroed(pid: &u32) -> Self {
        ZProcess {
            pid: *pid,
            uid: 0,
            cpu: 0.0,
            memory: 0,
            command: vec![],
            virt_mem: 0,
            starttime: 0,
            status: ProcessStatus::Dead,
        }
    }
}

// ordering implementations boilerplate because derive kinda sucks
impl PartialEq for ZProcess {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
    }
}

impl Eq for ZProcess {}

impl Ord for ZProcess {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pid.cmp(&self.pid)
    }
}

impl PartialOrd for ZProcess {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


enum ZProcessNode {
    ChildrenP {
        children: HashMap<u64, ZProcess>,
        node_val: ZProcess,
    },
    ChildLessP {
        node_val: ZProcess,
    }
}

use ZProcessNode::{ChildrenP,ChildLessP};

impl ZProcessNode {
    fn new_children(proc: ZProcess) -> Self {
        ChildrenP {
            children: HashMap::new(),
            node_val: proc,
            // node_val: ZProcess::new(pid),
        } 
    }
}

impl PartialEq for ZProcessNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&ZProcessNode::ChildrenP {node_val: ref v1,..}, &ZProcessNode::ChildrenP {node_val: ref v2,..})  => v1 == v2, 
            (&ZProcessNode::ChildLessP {node_val: ref v1}, &ZProcessNode::ChildLessP { node_val: ref v2}) => v1 == v2,
            (&ZProcessNode::ChildrenP { node_val: ref v1,.. }, &ZProcessNode::ChildLessP { node_val: ref v2 }) 
               | (&ZProcessNode::ChildLessP { node_val: ref v1 }, &ZProcessNode::ChildrenP { node_val: ref v2,..}) => v1 == v2,
        }
    }
}

impl Eq for ZProcessNode {}

impl Ord for ZProcessNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (&ChildrenP {node_val: ref v1,..}, &ChildrenP {node_val: ref v2,..})  => v1.cmp(v2), 
            (&ChildLessP {node_val: ref v1}, &ChildLessP { node_val: ref v2}) => v1.cmp(v2) ,
            (&ChildrenP { node_val: ref v1,.. }, &ChildLessP { node_val: ref v2 }) 
                | (&ChildLessP { node_val: ref v1 }, &ChildrenP { node_val: ref v2,..}) => v1.cmp(v2),
        }
    }
}

impl PartialOrd for ZProcessNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


/* the reason for this is that most subprocesses at least in linux systems
 * rarely exceed a depth of more than 3-5 so search is fairly fast
 * and the reason for storing subtrees in a list instead of just having a 
 * big tree is mainly for binary search so usually finding a process within 
 * a node will be O(log(n)+k) and finding a root node O(log(n))
 *  */
struct ProcessSubtrees {
    subtree_vec: Vec<ZProcessNode>,
}

impl ProcessSubtrees {
    fn new() -> Self{
        ProcessSubtrees {
            subtree_vec: vec![]
        }
    }

    fn find_root_sorted(&self, search_pid: &u32) -> Option<&ZProcessNode>{
        let temp_node = ZProcessNode::ChildLessP { 
            node_val: ZProcess::from_pid_as_zeroed(search_pid)
        };

        match self.subtree_vec.binary_search_by(|idx| idx.cmp(&temp_node)) {
            Ok(found_index) => { Some(&self.subtree_vec[found_index]) },
            Err(_) => { return None },
        }
    }

    fn find_subnode(search_pid: &u32) {
        let temp_node = ZProcessNode::ChildLessP { 
            node_val: ZProcess::from_pid_as_zeroed(search_pid) 
        };

        for 
    }

    fn pop_subtree() {

    }


} 