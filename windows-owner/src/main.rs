#![cfg_attr(windows,windows_subsystem="windows")]
#[cfg(not(windows))]fn main(){eprintln!("Windows only");}
#[cfg(windows)]mod gui{
use std::{ffi::c_void,ptr,mem,cell::RefCell,path::PathBuf,os::windows::ffi::OsStringExt};
use computer_link_owner_proof::{Owner,snapshot};
type H=*mut c_void;
#[repr(C)]struct Point{x:i32,y:i32}
#[repr(C)]struct Msg{hwnd:H,msg:u32,w:usize,l:isize,time:u32,point:Point,private:u32}
#[repr(C)]struct Class{size:u32,style:u32,proc:Option<unsafe extern "system" fn(H,u32,usize,isize)->isize>,cls:i32,win:i32,instance:H,icon:H,cursor:H,brush:H,menu:*const u16,name:*const u16,small:H}
#[repr(C)]struct Nid{size:u32,hwnd:H,id:u32,flags:u32,callback:u32,icon:H,tip:[u16;128],state:u32,mask:u32,info:[u16;256],version:u32,title:[u16;64],infoflags:u32,guid:[u8;16],balloon:H}
#[repr(C)]struct Ofn{size:u32,owner:H,instance:H,filter:*const u16,custom:*mut u16,max_custom:u32,index:u32,file:*mut u16,max_file:u32,file_title:*mut u16,max_title:u32,initial:*const u16,title:*const u16,flags:u32,offset:u16,extension:u16,defext:*const u16,data:isize,hook:H,template:*const u16,reserved:H,reserved2:u32,flags_ex:u32}
#[link(name="user32")]unsafe extern "system"{
fn RegisterClassExW(c:*const Class)->u16;fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,parent:H,menu:H,instance:H,param:H)->H;
fn DefWindowProcW(h:H,m:u32,w:usize,l:isize)->isize;fn GetMessageW(m:*mut Msg,h:H,min:u32,max:u32)->i32;fn TranslateMessage(m:*const Msg)->i32;fn DispatchMessageW(m:*const Msg)->isize;fn PostQuitMessage(code:i32);fn DestroyWindow(h:H)->i32;fn ShowWindow(h:H,n:i32)->i32;fn SetForegroundWindow(h:H)->i32;
fn MessageBoxW(h:H,text:*const u16,title:*const u16,flags:u32)->i32;fn CreateMenu()->H;fn AppendMenuW(menu:H,flags:u32,id:usize,text:*const u16)->i32;fn SetMenu(h:H,menu:H)->i32;fn SetWindowTextW(h:H,text:*const u16)->i32;fn LoadIconW(i:H,name:*const u16)->H;
}
#[link(name="kernel32")]unsafe extern "system"{fn GetModuleHandleW(n:*const u16)->H;fn ExitProcess(code:u32)->!;}
#[link(name="shell32")]unsafe extern "system"{fn Shell_NotifyIconW(m:u32,n:*const Nid)->i32;}
#[link(name="comdlg32")]unsafe extern "system"{fn GetOpenFileNameW(o:*mut Ofn)->i32;}
struct State{owner:Owner,selected:Option<(PathBuf,Vec<u8>)>,tray:Option<Nid>}
thread_local!{static STATE:RefCell<State>=RefCell::new(State{owner:Owner::default(),selected:None,tray:None});}
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
fn notice(h:H,s:&str){unsafe{MessageBoxW(h,wide(s).as_ptr(),wide("Computer-Link local proof").as_ptr(),0x40);}}
fn choose(h:H)->Option<PathBuf>{let mut b=vec![0u16;32768];let mut o:Ofn=unsafe{mem::zeroed()};o.size=mem::size_of::<Ofn>() as u32;o.owner=h;o.file=b.as_mut_ptr();o.max_file=b.len() as u32;o.flags=0x80000|0x1000|0x800|0x8|0x100000; // Explorer, file/path must exist, no cwd change, no link dereference.
if unsafe{GetOpenFileNameW(&mut o)}==0{return None}let n=b.iter().position(|v|*v==0)?;Some(PathBuf::from(std::ffi::OsString::from_wide(&b[..n])))}
fn stop(h:H)->bool{let result=STATE.with(|s|s.borrow_mut().owner.disconnect());match result{Ok(())=>{unsafe{SetWindowTextW(h,wide("Computer-Link — disconnected (local proof)").as_ptr());}true},Err(e)=>{notice(h,&format!("Disconnect could not confirm cleanup: {e}. Exit ends this process; do not start another session."));false}}}
fn exit(h:H){let clean=stop(h);STATE.with(|s|{if let Some(n)=s.borrow_mut().tray.take(){unsafe{Shell_NotifyIconW(2,&n);}}});if !clean{unsafe{ExitProcess(2)}}unsafe{DestroyWindow(h);}}
unsafe extern "system" fn procedure(h:H,m:u32,w:usize,l:isize)->isize{
match m{
0x111=>{match w&0xffff{
1=>{if stop(h){STATE.with(|s|s.borrow_mut().selected=None);if let Some(path)=choose(h){match snapshot(&path){Ok(bytes)=>{let count=bytes.len();STATE.with(|s|s.borrow_mut().selected=Some((path.clone(),bytes)));notice(h,&format!("Selected {} bytes from {}. Nothing shared yet. Allow once grants this captured text only, not future file changes.",count,path.display()));},Err(e)=>notice(h,&format!("Selection refused: {e}"))}}}},
2|3=>{let selected=STATE.with(|s|s.borrow().selected.clone());if w&0xffff==3&&selected.is_none(){notice(h,"Choose a bounded UTF-8 file first.");return 0}
let detail=if w&0xffff==3{format!("Allow ONE local paired read of the captured {} bytes from {}?",selected.as_ref().unwrap().1.len(),selected.as_ref().unwrap().0.display())}else{"Allow ONE local paired status request?".to_owned()};
let text=format!("{detail}\nExpires after 60 seconds. No write, delete, browser, network or administrator authority. Disconnect or Exit revokes this session. This proof has no external client pairing UI and does NOT connect ChatGPT.");
if unsafe{MessageBoxW(h,wide(&text).as_ptr(),wide("Explicit local permission").as_ptr(),0x104)}==6{
let bytes=if w&0xffff==3{selected.map(|s|s.1)}else{None};let result=STATE.with(|s|s.borrow_mut().owner.allow_once(bytes));match result{Ok(_internal_pairing)=>{unsafe{SetWindowTextW(h,wide("Computer-Link — one local grant (60s maximum)").as_ptr());}},Err(e)=>notice(h,&e.to_string())}
}},
4=>{stop(h);},5=>exit(h),_=>{}}0},
0x8001=>{if l as u32==0x202||l as u32==0x205{unsafe{ShowWindow(h,9);SetForegroundWindow(h);}}0},
0x10=>{exit(h);0},0x2=>{unsafe{PostQuitMessage(0);}0},_=>unsafe{DefWindowProcW(h,m,w,l)}}}
pub fn run(){unsafe{let instance=GetModuleHandleW(ptr::null());let name=wide("ComputerLinkOwnerProof");let icon=LoadIconW(ptr::null_mut(),32512usize as *const u16);let mut c:Class=mem::zeroed();c.size=mem::size_of::<Class>() as u32;c.proc=Some(procedure);c.instance=instance;c.name=name.as_ptr();c.icon=icon;c.brush=6usize as H;if RegisterClassExW(&c)==0{return}
let h=CreateWindowExW(0,name.as_ptr(),wide("Computer-Link — disconnected (local proof)").as_ptr(),0x10cf0000,100,100,760,220,ptr::null_mut(),ptr::null_mut(),instance,ptr::null_mut());if h.is_null(){return}
let menu=CreateMenu();for(id,label)in[(1,"Choose text file"),(2,"Allow status once"),(3,"Allow selected read once"),(4,"Disconnect"),(5,"Exit")]{AppendMenuW(menu,0,id,wide(label).as_ptr());}SetMenu(h,menu);
let mut n:Nid=mem::zeroed();n.size=mem::size_of::<Nid>() as u32;n.hwnd=h;n.id=1;n.flags=7;n.callback=0x8001;n.icon=icon;let tip=wide("Computer-Link local proof: click to open, Exit disconnects");n.tip[..tip.len()].copy_from_slice(&tip);if Shell_NotifyIconW(0,&n)!=0{STATE.with(|s|s.borrow_mut().tray=Some(n));}
let mut msg:Msg=mem::zeroed();while GetMessageW(&mut msg,ptr::null_mut(),0,0)>0{TranslateMessage(&msg);DispatchMessageW(&msg);} }}
}
#[cfg(windows)]fn main(){gui::run();}
