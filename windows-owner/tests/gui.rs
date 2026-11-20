#![cfg(windows)]
//! Actual GUI process/window/dialog acceptance on a disposable Windows CI desktop.
//! Win32 control messages are automation, not pixel/tray-click accessibility QA.
use std::{ffi::c_void,process::{Command,Child},thread,time::{Duration,Instant},ptr};
type H=*mut c_void;
#[link(name="user32")]unsafe extern "system"{
fn EnumWindows(cb:Option<unsafe extern "system" fn(H,isize)->i32>,data:isize)->i32;
fn GetWindowThreadProcessId(h:H,p:*mut u32)->u32;
fn GetClassNameW(h:H,b:*mut u16,n:i32)->i32;
fn GetWindowTextW(h:H,b:*mut u16,n:i32)->i32;
fn PostMessageW(h:H,m:u32,w:usize,l:isize)->i32;
fn GetDlgItem(h:H,id:i32)->H;
}
struct Search{pid:u32,class:&'static str,result:H}
unsafe extern "system" fn visit(h:H,data:isize)->i32{let s=unsafe{&mut *(data as *mut Search)};let mut pid=0;unsafe{GetWindowThreadProcessId(h,&mut pid);}if pid==s.pid{let mut b=[0u16;128];let n=unsafe{GetClassNameW(h,b.as_mut_ptr(),128)};if n>0&&String::from_utf16_lossy(&b[..n as usize])==s.class{s.result=h;return 0}}1}
fn window(pid:u32,class:&'static str)->H{let mut s=Search{pid,class,result:ptr::null_mut()};unsafe{EnumWindows(Some(visit),&mut s as *mut _ as isize);}s.result}
fn wait_window(child:&mut Child,class:&'static str)->H{let end=Instant::now()+Duration::from_secs(10);loop{assert!(child.try_wait().unwrap().is_none(),"GUI exited before {class}");let h=window(child.id(),class);if !h.is_null(){return h}assert!(Instant::now()<end,"no actual {class} window on this desktop");thread::sleep(Duration::from_millis(20));}}
fn title(h:H)->String{let mut b=[0u16;512];let n=unsafe{GetWindowTextW(h,b.as_mut_ptr(),512)};String::from_utf16_lossy(&b[..n.max(0) as usize])}
fn post(h:H,m:u32,w:usize,l:isize){assert_ne!(unsafe{PostMessageW(h,m,w,l)},0);}
struct Guard(Child);impl Drop for Guard{fn drop(&mut self){if self.0.try_wait().ok().flatten().is_none(){let _=self.0.kill();let _=self.0.wait();}}}
fn wait_title(child:&mut Child,h:H,part:&str){let end=Instant::now()+Duration::from_secs(5);while !title(h).contains(part){assert!(child.try_wait().unwrap().is_none());assert!(Instant::now()<end,"title did not contain {part}: {}",title(h));thread::sleep(Duration::from_millis(20));}}
#[test]fn actual_window_denial_grant_disconnect_exit_and_restart(){
for round in 0..2{
 let mut child=Guard(Command::new(env!("CARGO_BIN_EXE_computer-link-owner-proof")).spawn().unwrap());let h=wait_window(&mut child.0,"ComputerLinkOwnerProof");assert!(title(h).contains("disconnected"));
 // Real owner command opens an actual native MessageBox. Denying leaves no grant.
 post(h,0x111,2,0);let dialog=wait_window(&mut child.0,"#32770");let no=unsafe{GetDlgItem(dialog,7)};assert!(!no.is_null());post(no,0xf5,0,0);
 let deadline=Instant::now()+Duration::from_secs(5);while !window(child.0.id(),"#32770").is_null(){assert!(Instant::now()<deadline);thread::sleep(Duration::from_millis(20));}assert!(title(h).contains("disconnected"));
 // A second deliberate command and Yes are required. No hidden CLI consent flag.
 post(h,0x111,2,0);let dialog=wait_window(&mut child.0,"#32770");let yes=unsafe{GetDlgItem(dialog,6)};assert!(!yes.is_null());post(yes,0xf5,0,0);wait_title(&mut child.0,h,"one local grant");
 post(h,0x111,4,0);wait_title(&mut child.0,h,"disconnected");
 if round==0{post(h,0x111,5,0)}else{post(h,0x10,0,0)}
 let end=Instant::now()+Duration::from_secs(5);loop{if let Some(status)=child.0.try_wait().unwrap(){assert!(status.success());break}assert!(Instant::now()<end,"actual GUI process did not exit");thread::sleep(Duration::from_millis(20));}
 assert!(window(child.0.id(),"ComputerLinkOwnerProof").is_null());
}
}

fn prepare_status(child:&mut Child,h:H)->String{post(h,0x111,2,0);let dialog=wait_window(child,"#32770");post(unsafe{GetDlgItem(dialog,6)},0xf5,0,0);wait_title(child,h,"one local grant");title(h).split(" | ").nth(1).expect("public rendezvous name").to_owned()}
fn finish_client(child:&mut Child,success:bool){let end=Instant::now()+Duration::from_secs(5);loop{if let Some(status)=child.try_wait().unwrap(){assert_eq!(status.success(),success);break}assert!(Instant::now()<end,"client did not terminate");thread::sleep(Duration::from_millis(20));}}
#[test]fn actual_gui_authorizes_separate_client_and_invalidates_late_consent(){
 let mut owner=Guard(Command::new(env!("CARGO_BIN_EXE_computer-link-owner-proof")).spawn().unwrap());let h=wait_window(&mut owner.0,"ComputerLinkOwnerProof");
 for case in ["deny","allow","wrong-op","late-yes","peer-exit"]{
 let name=prepare_status(&mut owner.0,h);let mut client=Guard(Command::new(env!("CARGO_BIN_EXE_local-client")).arg(owner.0.id().to_string()).arg(&name).arg(if case=="wrong-op"{"read"}else{"status"}).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap());
 let prompt=wait_window(&mut owner.0,"#32770");assert_eq!(title(prompt),"Authorize connected client");
 if case=="peer-exit"{client.0.kill().unwrap();client.0.wait().unwrap();}
 if case=="late-yes"{post(h,0x111,4,0);wait_title(&mut owner.0,h,"disconnected");}
 post(unsafe{GetDlgItem(prompt,if case=="deny"{7}else{6})},0xf5,0,0);
 if case!="peer-exit"{finish_client(&mut client.0,case=="allow");}
 let mut out=String::new();use std::io::Read;client.0.stdout.take().unwrap().read_to_string(&mut out).unwrap();if case=="allow"{assert_eq!(out.trim(),"authenticated response: 1 bytes")}else{assert!(out.is_empty())}
 post(h,0x111,4,0);wait_title(&mut owner.0,h,"disconnected");
 // A completed/denied/revoked session must never accept another connecting client.
 let mut again=Guard(Command::new(env!("CARGO_BIN_EXE_local-client")).args([&owner.0.id().to_string(),&name,"status"]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().unwrap());finish_client(&mut again.0,false);
 }
 post(h,0x111,5,0);finish_client(&mut owner.0,true);
}
