#![cfg(windows)]
use std::{io,thread::{self,JoinHandle},time::{Duration,Instant},path::Path,fs::OpenOptions,os::windows::fs::{OpenOptionsExt,MetadataExt},io::Read};
use computer_link_local_transport::{Server,Stop,PeerInfo};
#[path="../../windows-workspace/src/root_scope.rs"] mod root_scope;
/// Read a bounded immutable snapshot through a non-reparse single-link handle.
/// Trusted/quiescent local ancestors only; not a hostile filesystem sandbox.
pub fn snapshot(path:&Path)->io::Result<Vec<u8>>{
 let _pins=root_scope::pin_parents(path)?;
 let f=OpenOptions::new().read(true).share_mode(1).custom_flags(0x00200000).open(path)?;
 let m=f.metadata()?;if !m.is_file()||m.file_attributes()&0x400!=0||m.file_size()>4096{return Err(io::Error::other("bounded regular file required"))}
 // Handle link count uses the same Windows API as the independently tested workspace.
 #[repr(C)]struct Info{attrs:u32,times:[u32;6],volume:u32,high:u32,low:u32,links:u32,index:[u32;2]}
 unsafe extern "system"{fn GetFileInformationByHandle(h:*mut std::ffi::c_void,i:*mut Info)->i32;}
 use std::os::windows::io::AsRawHandle;
 let mut i:Info=unsafe{std::mem::zeroed()};if unsafe{GetFileInformationByHandle(f.as_raw_handle(),&mut i)}==0||i.links!=1{return Err(io::Error::other("single-link handle required"))}
 let mut b=vec![];f.take(4097).read_to_end(&mut b)?;if b.len()>4096||std::str::from_utf8(&b).is_err(){return Err(io::Error::other("bounded UTF8 text required"))}Ok(b)
}
pub struct Owner{active:Option<(Stop,JoinHandle<io::Result<bool>>)>,fenced:bool}
impl Default for Owner{fn default()->Self{Self{active:None,fenced:false}}}
impl Owner{
 /// Internal handoff only. The native UI never prints or persists the bearer.
 pub fn allow_once(&mut self,snapshot:Option<Vec<u8>>)->io::Result<(String,[u8;32])>{
 if self.active.is_some()||self.fenced{return Err(io::Error::other("disconnect before another grant"))}
 if let Some(b)=&snapshot{if b.len()>4096||std::str::from_utf8(b).is_err(){return Err(io::Error::other("snapshot refused"))}}
 let server=Server::new(Duration::from_secs(60))?;let pairing=server.pairing();let stop=server.stop();
 let worker=thread::Builder::new().name("computer-link-local-session".into()).spawn(move||match snapshot{Some(b)=>server.serve_snapshot(b),None=>server.serve_once()})?;self.active=Some((stop,worker));Ok(pairing)
 }
 pub fn allow_paired<F>(&mut self,snapshot:Option<Vec<u8>>,approve:F)->io::Result<String> where F:FnOnce(PeerInfo,Instant,Stop)->io::Result<bool>+Send+'static{
 if self.active.is_some()||self.fenced{return Err(io::Error::other("disconnect before another grant"))}
 if let Some(b)=&snapshot{if b.len()>4096||std::str::from_utf8(b).is_err(){return Err(io::Error::other("snapshot refused"))}}
 let server=Server::new(Duration::from_secs(60))?;let name=server.pairing().0;let stop=server.stop();let worker=thread::Builder::new().name("computer-link-paired-session".into()).spawn(move||server.serve_paired(snapshot,approve))?;self.active=Some((stop,worker));Ok(name)
 }
 pub fn disconnect(&mut self)->io::Result<()>{
 let Some((stop,worker))=self.active.as_ref()else{return if self.fenced{Err(io::Error::other("shutdown remains unconfirmed; exit process required"))}else{Ok(())}};self.fenced=true;stop.revoke()?;
 let deadline=Instant::now()+Duration::from_secs(2);while !worker.is_finished(){if Instant::now()>=deadline{return Err(io::Error::other("shutdown unconfirmed; exit process required"))}thread::sleep(Duration::from_millis(5));}
 let (_,worker)=self.active.take().unwrap();match worker.join(){Err(_)=>return Err(io::Error::other("worker panicked; exit required")),Ok(Err(e)) if e.to_string().contains("process must terminate")=>return Err(e),_=>{}}
 self.fenced=false;Ok(())
 }
}
impl Drop for Owner{fn drop(&mut self){let _=self.disconnect();}}
#[cfg(test)]mod tests{
 use super::*;use std::io::{Read,Write};
 #[test]fn consent_snapshot_roundtrip_disconnect_restart(){let mut o=Owner::default();let(name,token)=o.allow_once(Some(b"approved snapshot".to_vec())).unwrap();assert!(o.allow_once(None).is_err());let mut f=OpenOptions::new().read(true).write(true).open(&name).unwrap();let mut r=token.to_vec();r.push(2);f.write_all(&r).unwrap();let mut len=[0;4];f.read_exact(&mut len).unwrap();let mut b=vec![0;u32::from_be_bytes(len) as usize];f.read_exact(&mut b).unwrap();assert_eq!(b,b"approved snapshot");drop(f);o.disconnect().unwrap();assert!(OpenOptions::new().read(true).write(true).open(name).is_err());o.allow_once(None).unwrap();o.disconnect().unwrap();}
 #[test]fn uncertain_completion_stays_fenced_across_repeated_disconnect(){let mut o=Owner::default();let server=Server::new(Duration::from_secs(3)).unwrap();let stop=server.stop();drop(server);o.active=Some((stop,thread::spawn(||Err(io::Error::other("cancel unconfirmed; process must terminate before restart")))));assert!(o.disconnect().is_err());assert!(o.active.is_none());assert!(o.fenced);assert!(o.disconnect().unwrap_err().to_string().contains("remains unconfirmed"));assert!(o.allow_once(None).is_err());}
 #[test]fn actual_file_snapshot_is_bounded_and_immutable(){let dir=std::env::temp_dir().join(format!("owner-snapshot-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir(&dir).unwrap();let file=dir.join("chosen.txt");std::fs::write(&file,b"first").unwrap();let captured=snapshot(&file).unwrap();std::fs::write(&file,b"second").unwrap();assert_eq!(captured,b"first");std::fs::write(&file,vec![0;4097]).unwrap();assert!(snapshot(&file).is_err());std::fs::write(&file,[255]).unwrap();assert!(snapshot(&file).is_err());std::fs::write(&file,b"valid").unwrap();std::fs::hard_link(&file,dir.join("alias.txt")).unwrap();assert_eq!(snapshot(&file).unwrap_err().to_string(),"single-link handle required");assert_eq!(std::fs::read(&file).unwrap(),b"valid");std::fs::remove_dir_all(dir).unwrap();}
 #[test]fn invalid_snapshot_creates_no_session(){let mut o=Owner::default();assert!(o.allow_once(Some(vec![0;4097])).is_err());assert!(o.active.is_none());assert!(o.allow_once(Some(vec![255])).is_err());}
}
