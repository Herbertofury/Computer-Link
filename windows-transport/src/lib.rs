//! Experimental local Windows status transport. No filesystem or desktop capability.
#![cfg(windows)]
use std::{ffi::c_void,io,ptr,mem,sync::Arc,time::{Duration,Instant}};
type H=*mut c_void;
#[repr(C)]struct SA{len:u32,sd:H,inherit:i32}
#[repr(C)]struct Ov{internal:usize,high:usize,offset:u32,offset_high:u32,event:H}
#[link(name="kernel32")]unsafe extern "system"{
 fn CloseHandle(h:H)->i32;fn GetCurrentProcess()->H;fn LocalFree(p:H)->H;
 fn CreateEventW(sa:*const SA,manual:i32,initial:i32,name:*const u16)->H;fn SetEvent(h:H)->i32;
 fn WaitForMultipleObjects(n:u32,h:*const H,all:i32,ms:u32)->u32;fn WaitForSingleObject(h:H,ms:u32)->u32;
 fn CreateNamedPipeW(n:*const u16,open:u32,mode:u32,instances:u32,out:u32,input:u32,timeout:u32,sa:*const SA)->H;
 fn ConnectNamedPipe(h:H,ov:*mut Ov)->i32;fn ReadFile(h:H,b:*mut c_void,n:u32,got:*mut u32,ov:*mut Ov)->i32;
 fn WriteFile(h:H,b:*const c_void,n:u32,got:*mut u32,ov:*mut Ov)->i32;
 fn GetOverlappedResult(h:H,ov:*mut Ov,n:*mut u32,wait:i32)->i32;fn CancelIoEx(h:H,ov:*mut Ov)->i32;
}
#[link(name="advapi32")]unsafe extern "system"{
 fn OpenProcessToken(p:H,access:u32,t:*mut H)->i32;fn GetTokenInformation(t:H,class:u32,b:H,len:u32,need:*mut u32)->i32;
 fn ConvertSidToStringSidW(sid:H,s:*mut *mut u16)->i32;fn ConvertStringSecurityDescriptorToSecurityDescriptorW(s:*const u16,rev:u32,sd:*mut H,len:*mut u32)->i32;
}
#[link(name="bcrypt")]unsafe extern "system"{fn BCryptGenRandom(a:H,b:*mut u8,n:u32,flags:u32)->i32;}
struct Owned(H);unsafe impl Send for Owned{} unsafe impl Sync for Owned{}
impl Drop for Owned{fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
fn owned(h:H)->io::Result<Arc<Owned>>{if h.is_null()||h as isize == -1 {Err(io::Error::last_os_error())}else{Ok(Arc::new(Owned(h)))}}
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
fn event()->io::Result<Arc<Owned>>{owned(unsafe{CreateEventW(ptr::null(),1,0,ptr::null())})}
struct Local(H);impl Drop for Local{fn drop(&mut self){unsafe{LocalFree(self.0);}}}
fn user_sd()->io::Result<Local>{unsafe{
 let mut t=ptr::null_mut();if OpenProcessToken(GetCurrentProcess(),8,&mut t)==0{return Err(io::Error::last_os_error())}let token=owned(t)?;
 let mut size=0;GetTokenInformation(token.0,1,ptr::null_mut(),0,&mut size);if size==0||size>65536{return Err(io::Error::other("token size refused"))}
 let mut data=vec![0usize;(size as usize+mem::size_of::<usize>()-1)/mem::size_of::<usize>()];if GetTokenInformation(token.0,1,data.as_mut_ptr().cast(),size,&mut size)==0{return Err(io::Error::last_os_error())}
 let sid=*(data.as_ptr() as *const H);let mut text=ptr::null_mut();if ConvertSidToStringSidW(sid,&mut text)==0{return Err(io::Error::last_os_error())}let _text=Local(text.cast());let mut len=0;while *text.add(len)!=0{len+=1;if len>256{return Err(io::Error::other("sid size refused"))}}
 let sid=String::from_utf16(std::slice::from_raw_parts(text,len)).map_err(|_|io::Error::other("sid refused"))?;
 // Explicit protected DACL only current user, not Everyone/Anonymous default.
 let s=wide(&format!("D:P(A;;GA;;;{sid})"));let mut sd=ptr::null_mut();if ConvertStringSecurityDescriptorToSecurityDescriptorW(s.as_ptr(),1,&mut sd,ptr::null_mut())==0{return Err(io::Error::last_os_error())}Ok(Local(sd))
}}
#[derive(Clone)]pub struct Stop(Arc<Owned>);impl Stop{pub fn revoke(&self)->io::Result<()>{if unsafe{SetEvent(self.0.0)}==0{Err(io::Error::last_os_error())}else{Ok(())}}}
pub struct Server{pipe:Arc<Owned>,stop:Stop,name:String,token:[u8;32],end:Instant}
// Heap-owned buffers/OVERLAPPED remain alive until terminal completion. On an
// unconfirmed cancellation they are intentionally retained, never freed early.
struct Pending{pipe:Arc<Owned>,event:Arc<Owned>,ov:Ov,bytes:Vec<u8>}
impl Server{
 pub fn new(lifetime:Duration)->io::Result<Self>{
  if lifetime.is_zero()||lifetime>Duration::from_secs(60){return Err(io::Error::other("lifetime refused"))}
  let mut seed=[0u8;64];if unsafe{BCryptGenRandom(ptr::null_mut(),seed.as_mut_ptr(),64,2)}<0{return Err(io::Error::other("entropy failed"))}
  let suffix=seed[..32].iter().map(|b|format!("{b:02x}")).collect::<String>();let name=format!(r"\\.\pipe\computer-link-{suffix}");let mut token=[0;32];token.copy_from_slice(&seed[32..]);let sd=user_sd()?;
  let sa=SA{len:mem::size_of::<SA>() as u32,sd:sd.0,inherit:0};
  // FIRST_PIPE_INSTANCE | OVERLAPPED | DUPLEX, byte mode, REJECT_REMOTE_CLIENTS.
  let pipe=owned(unsafe{CreateNamedPipeW(wide(&name).as_ptr(),0x40080003,8,1,256,256,0,&sa)})?;
  Ok(Self{pipe,stop:Stop(event()?),name,token,end:Instant::now()+lifetime})
 }
 /// In-memory handoff to an explicitly selected client; not user consent UI.
 pub fn pairing(&self)->(String,[u8;32]){(self.name.clone(),self.token)}
 pub fn stop(&self)->Stop{self.stop.clone()}
 fn operation(&self,kind:u8,bytes:Vec<u8>)->io::Result<Vec<u8>>{
  if Instant::now()>=self.end||unsafe{WaitForSingleObject(self.stop.0.0,0)}==0{return Err(io::Error::other("session stopped"))}
  let event=event()?;let mut p=Box::new(Pending{pipe:self.pipe.clone(),ov:Ov{internal:0,high:0,offset:0,offset_high:0,event:event.0},event,bytes});let mut got=0;
  let ok=unsafe{match kind{0=>ConnectNamedPipe(p.pipe.0,&mut p.ov),1=>ReadFile(p.pipe.0,p.bytes.as_mut_ptr().cast(),p.bytes.len() as u32,&mut got,&mut p.ov),_=>WriteFile(p.pipe.0,p.bytes.as_ptr().cast(),p.bytes.len() as u32,&mut got,&mut p.ov)}};
  if ok!=0{if Instant::now()>=self.end||unsafe{WaitForSingleObject(self.stop.0.0,0)}==0{return Err(io::Error::other("session stopped"))}p.bytes.truncate(got as usize);return Ok(mem::take(&mut p.bytes))}
  if ok==0{let error=io::Error::last_os_error();if kind==0&&error.raw_os_error()==Some(535){return Ok(vec![])}if error.raw_os_error()!=Some(997){return Err(error)}}
  let ms=self.end.saturating_duration_since(Instant::now()).as_millis().min(60000) as u32;let handles=[self.stop.0.0,p.event.0];
  let signaled=unsafe{WaitForMultipleObjects(2,handles.as_ptr(),0,ms)};
  if signaled!=1{unsafe{CancelIoEx(p.pipe.0,&mut p.ov);}
   if unsafe{WaitForSingleObject(p.event.0,1000)}!=0{mem::forget(p);return Err(io::Error::other("cancel unconfirmed; process must terminate before restart"))}
   if unsafe{GetOverlappedResult(p.pipe.0,&mut p.ov,&mut got,0)}==0 && io::Error::last_os_error().raw_os_error()==Some(996){mem::forget(p);return Err(io::Error::other("cancel incomplete; process must terminate before restart"))}return Err(io::Error::other("session cancelled or expired"))
  }
  if unsafe{GetOverlappedResult(p.pipe.0,&mut p.ov,&mut got,0)}==0{let error=io::Error::last_os_error();if error.raw_os_error()==Some(996){mem::forget(p);return Err(io::Error::other("completion incomplete; process must terminate before restart"))}return Err(error)}
  if Instant::now()>=self.end||unsafe{WaitForSingleObject(self.stop.0.0,0)}==0{return Err(io::Error::other("session stopped"))}
  p.bytes.truncate(got as usize);Ok(mem::take(&mut p.bytes))
 }
 /// Exactly one fixed-size authenticated status/revoke request, then close.
 /// Opcode0 status, opcode1 revoke. No root/file/desktop operations admitted.
 pub fn serve_once(self)->io::Result<bool>{self.operation(0,vec![])?;let b=self.operation(1,vec![0;33])?;
  if b.len()!=33{return Err(io::Error::other("frame refused"))}let diff=b[..32].iter().zip(self.token).fold(0u8,|a,(x,y)|a|(x^y));if diff!=0{return Err(io::Error::other("unauthorized"))}
  match b[32]{0=>{if self.operation(2,vec![1])?.len()!=1{return Err(io::Error::other("short response write"))}Ok(false)},1=>{self.stop.revoke()?;Ok(true)},_=>Err(io::Error::other("capability refused"))}
 }
}
#[cfg(test)]mod tests{
 use super::*;use std::{fs::OpenOptions,io::{Read,Write},thread};
 #[test]fn actual_local_authenticated_status(){let s=Server::new(Duration::from_secs(3)).unwrap();let(name,token)=s.pairing();let t=thread::spawn(move||s.serve_once());let mut f=OpenOptions::new().read(true).write(true).open(name).unwrap();let mut request=token.to_vec();request.push(0);f.write_all(&request).unwrap();let mut reply=[0];f.read_exact(&mut reply).unwrap();assert_eq!(reply,[1]);drop(f);assert!(!t.join().unwrap().unwrap());}
 #[test]fn already_connected_buffered_request_completes(){let s=Server::new(Duration::from_secs(3)).unwrap();let(name,token)=s.pairing();let mut f=OpenOptions::new().read(true).write(true).open(name).unwrap();let mut request=token.to_vec();request.push(0);f.write_all(&request).unwrap();let start=Instant::now();let t=thread::spawn(move||s.serve_once());let mut reply=[0];f.read_exact(&mut reply).unwrap();assert_eq!(reply,[1]);assert!(!t.join().unwrap().unwrap());assert!(start.elapsed()<Duration::from_secs(2));}
 #[test]fn wrong_token_never_admitted(){let s=Server::new(Duration::from_secs(3)).unwrap();let(name,mut token)=s.pairing();token[0]^=1;let t=thread::spawn(move||s.serve_once());let mut f=OpenOptions::new().read(true).write(true).open(name).unwrap();let mut request=token.to_vec();request.push(0);f.write_all(&request).unwrap();assert_eq!(t.join().unwrap().unwrap_err().to_string(),"unauthorized");}
 #[test]fn owner_exit_cancels_pending_connection(){let s=Server::new(Duration::from_secs(3)).unwrap();let stop=s.stop();let start=Instant::now();let t=thread::spawn(move||s.serve_once());thread::sleep(Duration::from_millis(50));stop.revoke().unwrap();assert!(t.join().unwrap().is_err());assert!(start.elapsed()<Duration::from_secs(2));}
 #[test]fn lifetime_expires_without_client(){let s=Server::new(Duration::from_millis(100)).unwrap();let start=Instant::now();assert!(s.serve_once().is_err());assert!(start.elapsed()<Duration::from_secs(2));}
 #[test]fn peer_revoke_closes_session(){let s=Server::new(Duration::from_secs(3)).unwrap();let(name,token)=s.pairing();let t=thread::spawn(move||s.serve_once());let mut f=OpenOptions::new().read(true).write(true).open(&name).unwrap();let mut request=token.to_vec();request.push(1);f.write_all(&request).unwrap();assert!(t.join().unwrap().unwrap());drop(f);assert!(OpenOptions::new().read(true).write(true).open(name).is_err());}
}
