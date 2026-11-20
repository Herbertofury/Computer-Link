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
