use std::{
    fs,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};
use tempfile::tempdir;
#[test]
fn protocol_drives_spec_transition() {
    let d = tempdir().unwrap();
    fs::create_dir_all(d.path().join(".lmbrain/specs/ready")).unwrap();
    fs::write(
        d.path().join(".lmbrain/specs/ready/SPEC-001.md"),
        "---\nid: SPEC-001\nstatus: ready\n---\n",
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_lmbrain-mcp"))
        .current_dir(d.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    writeln!(input,"{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"spec_start\",\"arguments\":{{\"path\":\".lmbrain/specs/ready/SPEC-001.md\"}}}}}}").unwrap();
    drop(input);
    let mut out = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut out)
        .unwrap();
    child.wait().unwrap();
    assert!(out.contains("result"));
    assert!(d.path().join(".lmbrain/specs/working/SPEC-001.md").exists());
}
