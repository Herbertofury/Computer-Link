//! Local stdio client adapter. No listener, credentials, registration or remote access.
use computer_link_mcp_session::{Session, Outcome};
use std::io::{self, BufRead, Write};
fn frame(r: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut out = Vec::new();
    loop {
        let b = r.fill_buf()?;
        if b.is_empty() { return if out.is_empty() { Ok(None) } else { Err(io::Error::other("truncated frame")) }; }
        let n = b.iter().position(|&x| x == b'\n').map_or(b.len(), |p| p + 1);
        if out.len() + n > 8192 { return Err(io::Error::other("frame too large")); }
        out.extend_from_slice(&b[..n]); r.consume(n);
        if out.last() == Some(&b'\n') { return Ok(Some(out)); }
    }
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut session = if args == ["--status-only"] { Session::default() } else {
        #[cfg(windows)] {
            if args.len() != 3 || args[0] != "--owner-approved-snapshot" { return Err(io::Error::other("use --status-only or --owner-approved-snapshot PIPE OWNER_PID")); }
            let pid: u32 = args[2].parse().map_err(|_| io::Error::other("invalid owner PID"))?;
            if pid == 0 { return Err(io::Error::other("invalid owner PID")); }
            let pipe = args[1].clone();
            Session::with_local_snapshot(move || computer_link_local_transport::paired_client(&pipe, pid, 2))
        }
        #[cfg(not(windows))] { return Err(io::Error::other("only --status-only is available on this OS")); }
    };
    let mut input = io::stdin().lock(); let mut output = io::stdout().lock();
    while let Some(bytes) = frame(&mut input)? {
        match session.handle(&bytes) {
            Outcome::Reply(v) => { writeln!(output, "{v}")?; output.flush()?; }
            Outcome::Notification => (),
            Outcome::Rejected => break,
        }
    }
    session.revoke();
    Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn frame_boundaries() {
        assert_eq!(frame(&mut io::Cursor::new(b"{}\n")).unwrap(),Some(b"{}\n".to_vec()));
        assert!(frame(&mut io::Cursor::new(b"{}")).is_err());
        assert!(frame(&mut io::Cursor::new(vec![b'a';8193])).is_err());
        assert_eq!(frame(&mut io::Cursor::new(b"")).unwrap(),None);
    }
}
