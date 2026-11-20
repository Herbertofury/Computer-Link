#![cfg(windows)]
//! Real interactive owner-owned MCP, synthetic status only. No external host.
use computer_link_owner_proof::Owner;
use std::{io::{BufRead,BufReader,Write,Read},path::Path,sync::mpsc,time::Duration};
fn bridge()->&'static Path{Path::new(env!("CARGO_BIN_EXE_computer-link-mcp-session"))}
fn read_lines(output:std::fs::File)->mpsc::Receiver<Vec<u8>>{
 let(tx,rx)=mpsc::channel();std::thread::spawn(move||{
  let mut reader=BufReader::new(output);loop{
   let mut line=Vec::new();match reader.by_ref().take(8193).read_until(b'\n',&mut line){
    Ok(0)=>break,Ok(_) if line.len()<=8192&&line.ends_with(b"\n")=>{if tx.send(line).is_err(){break}},_=>break
   }
  }
 });rx
}
fn reply(rx:&mpsc::Receiver<Vec<u8>>)->serde_json::Value{serde_json::from_slice(&rx.recv_timeout(Duration::from_secs(5)).expect("bounded MCP response")).unwrap()}
#[test]fn actual_interactive_status_child_routes_requests_and_disconnect_breaks_io(){
 let mut owner=Owner::default();let mut io=owner.launch_local_status_session(bridge()).unwrap();
 assert!(owner.launch_local_status_session(bridge()).is_err());
 let rx=read_lines(io.output);
 writeln!(io.input,"{}",r#"{"jsonrpc":"2.0","id":17,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"local-owner-test","version":"1"}}}"#).unwrap();
 assert_eq!(reply(&rx)["id"],17);
 writeln!(io.input,"{}",r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).unwrap();
 writeln!(io.input,"{}",r#"{"jsonrpc":"2.0","id":29,"method":"tools/list"}"#).unwrap();
 let list=reply(&rx);assert_eq!(list["id"],29);
 assert!(!list.to_string().contains("read_owner_snapshot"));
 // Responses follow actual independent caller requests, not a fixed transcript.
 writeln!(io.input,"{}",r#"{"jsonrpc":"2.0","id":31,"method":"tools/call","params":{"name":"read_owner_snapshot","arguments":{}}}"#).unwrap();
 let denied=reply(&rx);assert_eq!(denied["id"],31);assert!(denied.get("error").is_some()||denied["result"]["isError"]==true);
 assert!(owner.local_worker_exit_code().unwrap().is_none());
 owner.disconnect().unwrap();
 assert!(matches!(rx.recv_timeout(Duration::from_secs(3)),Err(mpsc::RecvTimeoutError::Disconnected)),"stdout must close after Job termination");
 assert!(io.input.write_all(b"{}\n").is_err(),"stdin must not survive disconnect");
 owner.disconnect().unwrap();
}
#[test]fn actual_interactive_drop_terminates_waiting_child(){
 let mut owner=Owner::default();let io=owner.launch_local_status_session(bridge()).unwrap();let rx=read_lines(io.output);
 drop(owner);
 assert!(matches!(rx.recv_timeout(Duration::from_secs(3)),Err(mpsc::RecvTimeoutError::Disconnected)));
}
#[test]fn actual_interactive_eof_exits_without_restart(){
 let mut owner=Owner::default();let io=owner.launch_local_status_session(bridge()).unwrap();let rx=read_lines(io.output);
 drop(io.input);assert!(matches!(rx.recv_timeout(Duration::from_secs(3)),Err(mpsc::RecvTimeoutError::Disconnected)));
 // No automatic re-launch: the old worker remains owned until explicit cleanup.
 assert!(owner.launch_local_status_session(bridge()).is_err());owner.disconnect().unwrap();
}
