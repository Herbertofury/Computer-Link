#![cfg(windows)]
//! Test-only native file-dialog acceptance; no production source changes.
use std::{ffi::c_void,process::{Command,Child},time::{Duration,Instant},thread,ptr};
type H=*mut c_void;
#[link(name="user32")]unsafe extern "system"{
fn EnumWindows(cb:Option<unsafe extern "system" fn(H,isize)->i32>,d:isize)->i32;fn EnumChildWindows(h:H,cb:Option<unsafe extern "system" fn(H,isize)->i32>,d:isize)->i32;
fn GetWindowThreadProcessId(h:H,p:*mut u32)->u32;fn GetClassNameW(h:H,b:*mut u16,n:i32)->i32;fn GetWindowTextW(h:H,b:*mut u16,n:i32)->i32;fn GetDlgItem(h:H,id:i32)->H;fn GetDlgCtrlID(h:H)->i32;
fn GetWindowLongPtrW(h:H,index:i32)->isize;fn IsWindowEnabled(h:H)->i32;fn IsWindowVisible(h:H)->i32;
fn PostMessageW(h:H,m:u32,w:usize,l:isize)->i32;fn SendMessageTimeoutW(h:H,m:u32,w:usize,l:isize,f:u32,ms:u32,r:*mut usize)->isize;
}
#[link(name="kernel32")]unsafe extern "system"{fn SetLastError(code:u32);fn GetLastError()->u32;}
struct Find{pid:u32,class:&'static str,h:H}
unsafe extern "system" fn visit(h:H,d:isize)->i32{let f=unsafe{&mut*(d as *mut Find)};let mut pid=0;unsafe{GetWindowThreadProcessId(h,&mut pid);}let mut b=[0u16;128];let n=unsafe{GetClassNameW(h,b.as_mut_ptr(),128)};if pid==f.pid&&n>0&&String::from_utf16_lossy(&b[..n as usize])==f.class{f.h=h;return 0}1}
fn find(pid:u32,class:&'static str)->H{let mut f=Find{pid,class,h:ptr::null_mut()};unsafe{EnumWindows(Some(visit),&mut f as *mut _ as isize);}f.h}
fn text(h:H)->String{let mut b=[0u16;32768];let n=unsafe{GetWindowTextW(h,b.as_mut_ptr(),b.len() as i32)};String::from_utf16_lossy(&b[..n.max(0)as usize])}
struct Bodies{pid:u32,rows:Vec<String>,failed:bool}
unsafe extern "system" fn collect(h:H,d:isize)->i32{let state=unsafe{&mut*(d as *mut Bodies)};let mut pid=0;unsafe{GetWindowThreadProcessId(h,&mut pid);}if pid!=state.pid{return 1}let mut class=[0u16;32];let n=unsafe{GetClassNameW(h,class.as_mut_ptr(),32)};if n<=0||String::from_utf16_lossy(&class[..n as usize])!="Static"{return 1}
 let mut buffer=[0u16;32768];let mut count=0usize;let ok=unsafe{SendMessageTimeoutW(h,0xd,buffer.len(),buffer.as_mut_ptr() as isize,2,2000,&mut count)};if ok==0||count>=buffer.len(){state.failed=true;return 0}state.rows.push(String::from_utf16_lossy(&buffer[..count]));1}
fn content(h:H)->String{let mut pid=0;unsafe{GetWindowThreadProcessId(h,&mut pid);}assert_ne!(pid,0);let mut state=Bodies{pid,rows:vec![],failed:false};unsafe{EnumChildWindows(h,Some(collect),&mut state as *mut _ as isize);}assert!(!state.failed,"bounded cross-process WM_GETTEXT failed");state.rows.join("\n")}
struct Field{pid:u32,h:H,seen:Vec<(i32,String)>}
unsafe extern "system" fn filename(h:H,d:isize)->i32{let f=unsafe{&mut*(d as *mut Field)};let mut pid=0;unsafe{GetWindowThreadProcessId(h,&mut pid);}if pid!=f.pid{return 1}let mut b=[0u16;128];let n=unsafe{GetClassNameW(h,b.as_mut_ptr(),128)};if n<=0{return 1}let class=String::from_utf16_lossy(&b[..n as usize]);let id=unsafe{GetDlgCtrlID(h)};if f.seen.len()<100{f.seen.push((id,class.clone()));}if matches!(id,1148|1152)&&matches!(class.as_str(),"ComboBox"|"ComboBoxEx32"|"Edit")&&unsafe{IsWindowVisible(h)!=0&&IsWindowEnabled(h)!=0}{f.h=h;return 0}1}
unsafe extern "system" fn filename_edit(h:H,d:isize)->i32{let f=unsafe{&mut*(d as *mut Field)};let mut pid=0;unsafe{GetWindowThreadProcessId(h,&mut pid);}if pid!=f.pid{return 1}let mut b=[0u16;32];let n=unsafe{GetClassNameW(h,b.as_mut_ptr(),32)};if n>0&&String::from_utf16_lossy(&b[..n as usize])=="Edit"&&unsafe{IsWindowVisible(h)!=0&&IsWindowEnabled(h)!=0}{f.h=h;return 0}1}
fn editable(container:H,pid:u32)->H{let mut f=Field{pid,h:ptr::null_mut(),seen:vec![]};unsafe{filename_edit(container,&mut f as *mut _ as isize);}if f.h.is_null(){unsafe{EnumChildWindows(container,Some(filename_edit),&mut f as *mut _ as isize);}}f.h}
fn field(dialog:H,pid:u32)->Field{let mut f=Field{pid,h:ptr::null_mut(),seen:vec![]};unsafe{EnumChildWindows(dialog,Some(filename),&mut f as *mut _ as isize);}f}
fn post(h:H,m:u32,w:usize,l:isize){assert_ne!(unsafe{PostMessageW(h,m,w,l)},0)}
fn until(pid:u32,phase:&str,mut f:impl FnMut()->bool){let end=Instant::now()+Duration::from_secs(8);while !f(){if Instant::now()>=end{let d=find(pid,"#32770");let caption=if d.is_null(){String::new()}else{text(d)};let body=if d.is_null(){String::new()}else{content(d)};panic!("native dialog condition timed out: phase={phase}, dialog={}, owner_notice={}, permission={}, selected_notice={}, selection_refused={}, choose_required={}",!d.is_null(),caption=="Computer-Link local proof",caption=="Explicit local permission",body.contains("Nothing shared yet"),body.contains("Selection refused"),body.contains("Choose a bounded UTF-8 file first"));}thread::sleep(Duration::from_millis(20));}}
// On failure report identity/visibility metadata, never dialog text or paths.
struct Controls { pid: u32, count: usize }
unsafe extern "system" fn diagnose_control(h: H, data: isize) -> i32 {
    let state = unsafe { &mut *(data as *mut Controls) };
    if state.count >= 40 { return 0; }
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(h, &mut pid); }
    if pid != state.pid { return 1; }
    state.count += 1;
    let mut class = [0u16; 128];
    let n = unsafe { GetClassNameW(h, class.as_mut_ptr(), class.len() as i32) };
    let class = String::from_utf16_lossy(&class[..n.max(0) as usize]);
    eprintln!("notice child: id={} class={:?} visible={} enabled={}",
        unsafe { GetDlgCtrlID(h) }, class,
        unsafe { IsWindowVisible(h) }, unsafe { IsWindowEnabled(h) });
    1
}
fn dismiss_notice(pid:u32,dialog:H){
 let end=Instant::now()+Duration::from_secs(8);
 let button=loop {
    let button=unsafe{GetDlgItem(dialog,1)};let mut dp=0;let mut bp=0;
    unsafe{GetWindowThreadProcessId(dialog,&mut dp);GetWindowThreadProcessId(button,&mut bp);}
    let dv=unsafe{IsWindowVisible(dialog)};let de=unsafe{IsWindowEnabled(dialog)};
    let bv=unsafe{IsWindowVisible(button)};let be=unsafe{IsWindowEnabled(button)};
    if !button.is_null()&&dp==pid&&bp==pid&&dv!=0&&de!=0&&bv!=0&&be!=0 { break button; }
    if Instant::now()>=end {
        eprintln!("notice readiness failed: button_present={} dialog_pid_match={} button_pid_match={} dialog_visible={dv} dialog_enabled={de} button_visible={bv} button_enabled={be}", !button.is_null(), dp==pid, bp==pid);
        let mut controls=Controls{pid,count:0};
        unsafe{EnumChildWindows(dialog,Some(diagnose_control),&mut controls as *mut _ as isize);}
        panic!("notice IDOK control not actionable; assertions preserved");
    }
    thread::sleep(Duration::from_millis(20));
 };
 post(dialog,0x111,1,button as isize);
}
struct Guard(Child);impl Drop for Guard{fn drop(&mut self){if self.0.try_wait().ok().flatten().is_none(){let _=self.0.kill();let _=self.0.wait();}}}
#[test]fn actual_file_dialog_selection_and_cancel_preserve_fixture(){
 let dir=std::env::temp_dir().join(format!("owner-dialog-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir(&dir).unwrap();let path=dir.join("fixture.txt");std::fs::write(&path,b"native dialog fixture").unwrap();
 let mut owner=Guard(Command::new(env!("CARGO_BIN_EXE_computer-link-owner-proof")).spawn().unwrap());let pid=owner.0.id();until(pid,"owner-window",||!find(pid,"ComputerLinkOwnerProof").is_null());let h=find(pid,"ComputerLinkOwnerProof");
 post(h,0x111,1,0);until(pid,"choose-dialog",||!find(pid,"#32770").is_null());let dialog=find(pid,"#32770");let deadline=Instant::now()+Duration::from_secs(8);let field=loop{let f=field(dialog,pid);if !f.h.is_null(){let edit=editable(f.h,pid);if !edit.is_null(){break edit}}assert!(Instant::now()<deadline,"native filename control missing; class/id metadata only: {:?}",f.seen);thread::sleep(Duration::from_millis(20));};
 use std::os::windows::ffi::OsStrExt;let value:Vec<u16>=path.as_os_str().encode_wide().chain(Some(0)).collect();let mut result=0;unsafe{SetLastError(0);}let sent=unsafe{SendMessageTimeoutW(field,0xc,0,value.as_ptr() as isize,2,2000,&mut result)};let set_error=unsafe{GetLastError()};let mut observed=vec![0u16;32768];let mut length=0;unsafe{SetLastError(0);}let read=unsafe{SendMessageTimeoutW(field,0xd,observed.len(),observed.as_mut_ptr() as isize,2,2000,&mut length)};let read_error=unsafe{GetLastError()};let exact=length<observed.len()&&observed[..length]==value[..value.len()-1];
 // Bounded metadata only: no field contents, paths or secrets in diagnostics.
 eprintln!("filename diagnostics: sent={sent} result={result} set_error={set_error} read={read} read_error={read_error} count={length} exact={exact} id={} enabled={} visible={} style={:#x}",unsafe{GetDlgCtrlID(field)},unsafe{IsWindowEnabled(field)},unsafe{IsWindowVisible(field)},unsafe{GetWindowLongPtrW(field,-16)});
 assert_ne!(sent,0,"WM_SETTEXT dispatch failed");assert_ne!(result,0,"filename Edit refused WM_SETTEXT");assert_ne!(read,0,"WM_GETTEXT dispatch failed");assert!(length<observed.len());assert!(exact,"filename readback differs");post(dialog,0x111,1,0);
 until(pid,"selected-notice",||{let d=find(pid,"#32770");!d.is_null()&&text(d)=="Computer-Link local proof"&&content(d).contains("Nothing shared yet")});let notice=find(pid,"#32770");assert!(content(notice).contains("21 bytes"));dismiss_notice(pid,notice);until(pid,"selected-notice-dismissed",||find(pid,"#32770").is_null());
 // Selecting alone never opens a grant; an explicit read confirmation remains necessary.
 assert!(text(h).contains("disconnected"));post(h,0x111,3,0);until(pid,"read-permission",||!find(pid,"#32770").is_null());let permission=find(pid,"#32770");assert!(content(permission).contains("fixture.txt"));post(unsafe{GetDlgItem(permission,7)},0xf5,0,0);until(pid,"read-permission-dismissed",||find(pid,"#32770").is_null());assert!(text(h).contains("disconnected"));
 // Cancel a new selection; the prior choice must not silently remain selectable.
 post(h,0x111,1,0);until(pid,"reselection-dialog",||!find(pid,"#32770").is_null());post(find(pid,"#32770"),0x111,2,0);until(pid,"reselection-cancelled",||find(pid,"#32770").is_null());post(h,0x111,3,0);until(pid,"missing-selection-notice",||!find(pid,"#32770").is_null());let refusal=find(pid,"#32770");assert!(content(refusal).contains("Choose a bounded UTF-8 file first"));dismiss_notice(pid,refusal);until(pid,"missing-selection-dismissed",||find(pid,"#32770").is_null());
 post(h,0x111,5,0);until(pid,"owner-exited",||owner.0.try_wait().unwrap().is_some());assert!(owner.0.try_wait().unwrap().unwrap().success());assert_eq!(std::fs::read(&path).unwrap(),b"native dialog fixture");std::fs::remove_dir_all(dir).unwrap();
}
