#![allow(unused)]
use core::panic;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::vec::Vec;
use std::cmp::{Ord,Ordering};
use std::ops::Drop;

use sysinfo::{Process, ProcessStatus, Uid};

#[derive(Clone,Debug)]
pub struct ZProcess {
    // username: String,
    pub pid: u32,
    pub uid: u32,
    pub cpu: f32,
    pub memory: u64,
    pub command: Vec<String>,
    pub virt_mem: u64,
    pub starttime: u64,
    pub status: ProcessStatus,
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
    pub const fn from_pid_as_zeroed(pid: &u32) -> Self {
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


#[derive(Clone,Debug)]
pub enum TreeNode {
    WithChildren {
        children: HashMap<u32, TreeNode>,
        node_val: ZProcess,
    },
    WithoutChildren {
        node_val: ZProcess,
    }
}

use TreeNode::{WithChildren,WithoutChildren};

impl TreeNode {
    pub fn new_with(proc: ZProcess, children: HashMap<u64,TreeNode>) -> Self {
        WithChildren {
            children: HashMap::new(),
            node_val: proc,
        } 
    }

    pub const fn new_without(proc: ZProcess) -> Self {
        WithoutChildren { node_val: proc }
    }

    pub fn node(&mut self) -> &mut ZProcess {
        match self {
            TreeNode::WithChildren { node_val, ..} => node_val,
            TreeNode::WithoutChildren { node_val } => node_val,
        }
    }

    pub fn children(&mut self) -> Option<&mut HashMap<u32, TreeNode>> {
        match self {
            TreeNode::WithChildren { children, ..} => Some(children),
            TreeNode::WithoutChildren { .. } => None,
        }
    }

    fn has_children(&self) -> bool {
        match self {
            &TreeNode::WithoutChildren { .. } => false,
            &TreeNode::WithChildren { .. } => true,
        }
    }

    fn insert(&mut self, mut node: TreeNode, parent: Option<&ZProcess>) {
        match parent {
            Some(proc) => {
                self.look_up_children(proc).unwrap().insert(node.node().pid, node);
            }

            None => {
                self.children().unwrap().insert(node.node().pid, node);
            }
        }
    }

    pub fn look_up_process(
        &mut self, 
        target: &ZProcess) -> Option<&mut ZProcess> 
    {
        match self {
            WithChildren {
                children,
                node_val,
            } => {
                if node_val == target {
                    return Some(node_val);
                }
                for (k,v) in children.iter_mut() {
                    if *k == target.pid {
                        return Some(v.node());
                    } else {
                        if let TreeNode::WithoutChildren { .. } = *v {
                            continue;
                        } 
                        match v.look_up_process(target) {
                            Some(found_node) => { return Some(found_node); },
                            None => continue,
                        }
                    }
                }
                return None
            },

            WithoutChildren { 
                node_val
            } => {
                if node_val == target { 
                    return Some(self.node());
                } else {
                    return None
                }
            }
        }
    }

    fn look_up_children(
        &mut self,
        target: &ZProcess ) -> Option<&mut HashMap<u32, TreeNode>> 
    {
        match self {
            WithoutChildren { .. } => None,

            WithChildren { children, node_val} => {
                if node_val == target {
                    Some(children)
                } else {
                    for (k,v) in children.iter_mut().filter(|(k,v)| v.has_children()) {
                        if *k == target.pid {
                            return Some(v.children().unwrap())
                        }
                        if let Some(found_target) = v.look_up_children(target) {
                            return Some(found_target);
                        } 
                        continue;
                    } 
                    return None
                }
            }
        }
    }

}

impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&WithChildren {node_val: ref v1,..}, &WithChildren {node_val: ref v2,..})  => v1 == v2, 
            (&WithoutChildren {node_val: ref v1}, &WithoutChildren { node_val: ref v2}) => v1 == v2,
            (&WithChildren { node_val: ref v1,.. }, &WithoutChildren { node_val: ref v2 }) 
               | (&WithoutChildren { node_val: ref v1 }, &WithChildren { node_val: ref v2,..}) => v1 == v2,
        }
    }
}

impl Eq for TreeNode {}

impl Ord for TreeNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (&WithChildren {node_val: ref v1,..}, &WithChildren {node_val: ref v2,..})  => v1.cmp(v2), 
            (&WithoutChildren {node_val: ref v1}, &WithoutChildren { node_val: ref v2}) => v1.cmp(v2) ,
            (&WithChildren { node_val: ref v1,.. }, &WithoutChildren { node_val: ref v2 }) 
                | (&WithoutChildren { node_val: ref v1 }, &WithChildren { node_val: ref v2,..}) => v1.cmp(v2),
        }
    }
}

impl PartialOrd for TreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tree_methods_testing {
    use super::{TreeNode,ZProcess};
    use std::collections::HashMap;

    #[test]
    fn look_up_check() {
        let mut tree = TreeNode::new_with(ZProcess::from_pid_as_zeroed(&1000), HashMap::new());

        let children = tree.children().unwrap();
        children.insert(1002, TreeNode::WithoutChildren { node_val: ZProcess::from_pid_as_zeroed(&1002)});
        children.insert(1003, TreeNode::WithoutChildren { node_val: ZProcess::from_pid_as_zeroed(&1003)});
        children.insert(1004, TreeNode::WithoutChildren { node_val: ZProcess::from_pid_as_zeroed(&1004)}); 

        let test_result = tree.look_up_process(&ZProcess::from_pid_as_zeroed(&1004));

        let proc_for_assert = ZProcess::from_pid_as_zeroed(&1004);
        // assert_eq!(test_result, Some(TreeNode::WithoutChildren { node_val: proc_for_assert}));
    }

    #[test]
    fn test_mut() {
        let mut tree = TreeNode::new_with(ZProcess::from_pid_as_zeroed(&1000), HashMap::new());

        let children = tree.children().unwrap();
        children.insert(1002, TreeNode::WithoutChildren { node_val: ZProcess::from_pid_as_zeroed(&1002)});
        children.insert(1003, TreeNode::WithoutChildren { node_val: ZProcess::from_pid_as_zeroed(&1003)});
        children.insert(1004, TreeNode::WithoutChildren { node_val: ZProcess::from_pid_as_zeroed(&1004)}); 

        let mut proc = tree.look_up_process(&ZProcess::from_pid_as_zeroed(&1002)).unwrap();
        proc.memory = 50000;
        drop(proc);

        assert_eq!(50000 as u64, tree.look_up_process(&ZProcess::from_pid_as_zeroed(&1002)).unwrap().memory);
    }

    // #[test]
    // fn look_up_children() {
    //     let mut tree = TreeNode::new_with(ZProcess::from_pid_as_zeroed(&1000), HashMap::new());
    //     let children = tree.children().unwrap();
    //     children.insert(1002, )
    // }
}