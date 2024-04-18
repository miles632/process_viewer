#![allow(unused)]

pub mod process;

use process::{TreeNode,ZProcess};

use sysinfo::{
    System, Process,
};

fn main() {
    let mut sys_handle = System::new();

    let zprocesses = sys_handle.processes().into_iter().map(|(_,sys_proc)|{
        ZProcess::new(sys_proc)
    }).collect::<Vec<ZProcess>>();

    dbg!(zprocesses.len());
}
