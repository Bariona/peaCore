#![no_std]
#![no_main]

use alloc::string::String;
use user_lib::{console::getchar, fork, exec, waitpid}; 

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;  // '\n' Line Feed
const CR: u8 = 0x0du8;  // '\r' Carriage Return 
const BS: u8 = 0x08u8;  // BackSpace
const DEL:u8 = 0x7fu8;  // Delete

#[no_mangle]
pub fn main() -> i32 {
  println!("User Shell:");
  let mut line: String = String::new();
  print!("$ ");
  loop {
    let c = getchar();
    match c {
      LF | CR => {
        println!("");
        if !line.is_empty() {
          line.push('\0');
          let pid = fork();
          if pid == 0 {
            if exec(line.as_str()) == -1 {
              println!("Error when execve(\"{}\")", line);
              return -4;
            }
            unreachable!();
          } else {
            let mut exit_code: i32 = 0;
            let exit_pid = waitpid(pid as usize, &mut exit_code);
            assert_eq!(exit_pid, pid);
            println!("Shell: Process {} exited with code {}", pid, exit_code);
          }
          line.clear();
        }
        print!("$ ");
      } 
      BS | DEL => {
        if !line.is_empty() {
          print!("{}", BS as char);
          print!(" ");
          print!("{}", BS as char);
          line.pop();
        }
      }
      _ => {
        print!("{}", c as char);
        line.push(c as char);
      }
    }
  }
}