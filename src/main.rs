#![allow(unused)]

mod gui;
use gui::EguiApp;
use eframe::{egui, App, NativeOptions};


mod process_tree;
use process_tree::{TreeNode,ZProcess};
use sysinfo::{
    System, Process,
};

use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashMap};
use std::boxed::Box;

fn main() -> Result<(), eframe::Error>{
    let mut sys_handle = System::new_all();
    // collect all the process metadata 
    let mut zprocesses = sys_handle.processes().into_iter().map(|(_,sys_proc)|{
        ZProcess::new(sys_proc)
    }).collect::<Vec<ZProcess>>();
    zprocesses.sort();

    let mut proc_tree = tree_init(zprocesses.clone());
    let proc_tree = &mut proc_tree;

    loop {
        proc_tree.step_through_and_update(&mut sys_handle);
        proc_tree.pop_zombie_procs(&mut sys_handle);
        proc_tree.clone().flatten_tree_into_list(&mut zprocesses);
        proc_tree.push_new_procs(&mut sys_handle, &zprocesses);
        
        sleep(Duration::from_millis(200));
    }

    // let options = NativeOptions {
    //     ..Default::default()
    // };
    // let app_handle = EguiApp::default();
    // eframe::run_native(
    //     "Process Viewer",
    //     options,
    //     Box::new(|cc| Box::new(EguiApp::new(cc, zprocesses2))),
    // );

    Ok(())
}

// constructs the initial tree from the processes at that time
// which will then be updated
fn tree_init(mut zprocesses: Vec<ZProcess>) -> TreeNode {
    // make a deep copy of zprocesses to be used in
    // the _find_children method
    let zprocesses_cl = zprocesses.clone();

    let mut root_proc_idx = 0;
    for (idx, proc) in zprocesses.iter().enumerate() {
        if proc.pid == 1 {
            root_proc_idx = idx;
        }
    }
    if root_proc_idx == 0 {panic!()}// TODO: remove later
    
    let mut root_proc = TreeNode::new(
        zprocesses.swap_remove(root_proc_idx), 
        HashMap::new()
    );

    // transform process vec into tree
    _find_children_and_append(&mut root_proc, &zprocesses_cl);

    // dbg!(root_proc.node());
    // dbg!(root_proc.children_mut().unwrap().len());
    // dbg!(root_proc.children());

    root_proc
}

fn _find_children_and_append(
    target_node: &mut TreeNode, 
    zproc_vec: &Vec<ZProcess>
)
{
    // if target_node.children_mut().is_some() {
        let children_idxs = _find_children(&mut target_node.val, zproc_vec);
        if children_idxs.is_none() {
            return;
        }
        for idx in children_idxs.unwrap().into_iter() {
            let proc = zproc_vec[idx].clone();
            let mut proc = TreeNode::new(
                proc,
                HashMap::new()
            );
            _find_children_and_append(&mut proc, zproc_vec);
            target_node.child_procs.insert(proc.val.pid, proc);
        }
    // }

}

// returns vec of indices
fn _find_children(
    target_proc: &mut ZProcess, 
    zproc_vec: &Vec<ZProcess>
) -> Option<Vec<usize>> 
{ 
    let mut children = Vec::new();
    for (idx, proc) in zproc_vec.iter().enumerate() {
        if proc.ppid == None {continue;}
        if proc.ppid.unwrap() == target_proc.pid { // TODO: dont fucking unwrap here
            children.push(idx)
        }
    }

    if children.len() == 0 { 
        None 
    } else {
        Some(children)
    }
}
