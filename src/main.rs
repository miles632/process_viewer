#![allow(unused)]

pub mod process;

use std::collections::HashMap;

use process::{TreeNode,ZProcess};

use sysinfo::{
    System, Process,
};

fn main() {
    let mut sys_handle = System::new_all();

    let mut zprocesses = sys_handle.processes().into_iter().map(|(_,sys_proc)|{
        ZProcess::new(sys_proc)
    }).collect::<Vec<ZProcess>>();

    let mut root_proc_idx = 0;
    for (idx, proc) in zprocesses.iter().enumerate() {
        if proc.pid == 1 {
            root_proc_idx = idx;
        }
    }
    if root_proc_idx == 0 {panic!()}// TODO: remove later

    let mut root = TreeNode::new_with(
        zprocesses.swap_remove(root_proc_idx), 
        HashMap::new()
    );
}
