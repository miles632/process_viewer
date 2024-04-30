#![allow(unused)]
use core::panic;
use std::borrow::Borrow;
use std::collections::{HashMap, hash_map::Entry};
use std::thread;
use std::vec::Vec;
use std::cmp::{Ord,Ordering};
use std::ops::Drop;
use std::boxed::Box;

use egui::accesskit::Tree;
use sysinfo::{Process, Uid, System};
use sysinfo::ProcessStatus;
use sysinfo::ProcessStatus::*;

#[derive(Clone,Debug)]
pub struct ZProcess {
    name: String,
    pub pid: u32,
    pub ppid: Option<u32>,
    // pub uid: u32,
    pub cpu: f32,
    pub memory: u64,
    pub command: Vec<String>,
    pub virt_mem: u64,
    pub starttime: u64,
    pub status: ProcessStatus,
}

impl ZProcess {
    pub fn new(sys_proc: &Process) -> Self {
        let ppid;
        match sys_proc.parent() {
            Some(pid) => {
                ppid = Some(pid.as_u32())
            }
            None => {
                ppid = None;
            }
        }

        ZProcess {
            name: sys_proc.name().to_string(),
            pid: sys_proc.pid().as_u32(),
            ppid: ppid,
            // uid: *(sys_proc.user_id().unwrap()).clone(), // TODO: this is weird
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
            name: String::new(),
            pid: *pid,
            ppid: None,
            // uid: 0,
            cpu: 0.0,
            memory: 0,
            command: vec![],
            virt_mem: 0,
            starttime: 0,
            status: Dead,
        }
    }

    fn proc_refresh(&mut self, sys_handle: &mut System) -> Option<Self> {
        sysinfo::System::refresh_process(sys_handle, sysinfo::Pid::from_u32(self.pid));
        match sys_handle.process(sysinfo::Pid::from_u32(self.pid)) {
            Some(proc) => Some(ZProcess::new(proc)),
            None => None,
        }
    }
}

// ordering implementations boilerplate because derive kinda sucks
impl PartialEq for ZProcess {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
    }
}

impl Eq for ZProcess{}

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

type ProcMap = HashMap<u32, TreeNode>;
#[derive(Clone,Debug)]
pub struct TreeNode {
    pub val: ZProcess,
    pub child_procs: ProcMap,
}

impl TreeNode {
    pub fn new(proc: ZProcess, children: ProcMap) -> Self {
        TreeNode {
            val: proc, 
            child_procs: children,
        }
    }

    pub fn insert(
        &mut self, 
        mut node: TreeNode, 
        parent: Option<&ZProcess>
    ) 
    {
        match parent {
            Some(proc) => {
                // self.look_up_children(proc).unwrap().insert(node.val.pid, node);
                self.look_up_children(proc).expect("god damnit").insert(node.val.pid, node);
            }

            None => {
                self.child_procs.insert(node.val.pid, node);
            }
        }
    }

    pub fn look_up_process(
        &mut self, 
        target: &ZProcess
    ) -> Option<&mut ZProcess> 
    {
        if self.val == *target {
            return Some(&mut self.val);
        }

        for (_,proc) in self.child_procs.iter_mut() {
            return proc.look_up_process(target)
        }

        None
    }

    
    pub fn look_up_children(
        &mut self, 
        target: &ZProcess,
    ) -> Option<&mut ProcMap> {
        if self.val == *target {
            return Some(&mut self.child_procs);
        }

        for (_, proc) in self.child_procs.iter_mut() {
            return proc.look_up_children(target)
        }

        None
    }

    pub fn step_through_and_update(
        &mut self, 
        sys_handle: &mut System
    ) {
        // update root

        self.val.proc_refresh(sys_handle); 

        if !self.child_procs.is_empty() {
            for (_, proc) in self.child_procs
                .iter_mut()
                .filter(|(_,proc)| {
                    match proc.val.status {
                        Zombie|Dead|Sleep|UninterruptibleDiskSleep => false,
                        _ => true,
                    }
                }) {
                    proc.step_through_and_update(sys_handle);
            }
        }
    }

    pub fn pop_zombie_procs(
        &mut self, 
        sys: &mut System
    ) {
        self.child_procs.retain(|_, proc| {
            proc.val.status != Zombie
            ||
            proc.val.status != Dead
        });

        for (_, proc) in self.child_procs.iter_mut() {
            proc.pop_zombie_procs(sys);
        }
    }

    #[warn(unused)]
    pub fn push_new_procs(
        &mut self,
        sys: &mut System,
        proc_vec: &Vec<ZProcess>
    ) {
        let new_proc_vec = sys.processes().into_iter().map(|(_,proc)|{
            ZProcess::new(proc)
        }).collect::<Vec<ZProcess>>();

        let mut procs_to_be_appended = Vec::new();

        for proc in new_proc_vec.into_iter().filter(|proc| proc.status == Zombie) { 
            if !proc_vec.contains(&proc) {
                procs_to_be_appended.push(proc)
            }
        }

        let mut procs_to_be_appended = procs_to_be_appended.into_iter().map(|proc|{
            TreeNode::new(proc, HashMap::new())
        }).collect::<Vec<TreeNode>>();


        let mut procs_wo_parents = procs_to_be_appended.clone();

        procs_wo_parents.retain(|proc|{
            !procs_to_be_appended.contains(&TreeNode::new(
                ZProcess::from_pid_as_zeroed(&proc.val.ppid.unwrap()),
                HashMap::new()
            ))
        });

        for mut proc in procs_wo_parents.into_iter() {
            for subproc in procs_to_be_appended.iter_mut().filter(|subproc| subproc.val.ppid.unwrap() != proc.val.pid) {
                proc.child_procs.insert(subproc.val.pid, subproc.to_owned());
            }
            match self.look_up_children(&ZProcess::from_pid_as_zeroed(&proc.val.ppid.unwrap())) {
                Some(children) => {
                    children.insert(proc.val.pid, proc)
                },
                None => {
                    self.child_procs.insert(proc.val.pid, proc)
                },
            };
        }
    }

    pub fn flatten_tree_into_list(self, list: &mut Vec<ZProcess>) {
        list.push(self.val);

        for (_, proc) in self.child_procs.into_iter() {
            proc.flatten_tree_into_list(list)
        }
    }

}

impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl Eq for TreeNode {}

impl Ord for TreeNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.val.cmp(&other.val)
    }
}

impl PartialOrd for TreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.val.partial_cmp(&other.val) {
            Some(core::cmp::Ordering::Equal) => {
                return Some(Ordering::Equal)
            }
            ord => return ord,
        }
    }
}