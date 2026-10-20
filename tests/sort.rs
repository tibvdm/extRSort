use std::{
    io::Write,
    process::{Command, Stdio}
};

/// Runs the sorter on `input` with the given arguments and returns its stdout.
fn sorter(input: &[u8], args: &[&str]) -> Vec<u8> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sorter"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // From a thread, so a sorter that writes before it has read everything cannot block on a full pipe.
    let mut stdin = child.stdin.take().unwrap();
    let input = input.to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&input).unwrap());

    // A sorter still running after a minute is killed, so a sorter that hangs fails the test.
    let pid = child.id();
    let (done, finished) = std::sync::mpsc::channel::<()>();
    let watchdog = std::thread::spawn(move || {
        if finished.recv_timeout(std::time::Duration::from_secs(60)).is_err() {
            let _ = Command::new("kill").arg(pid.to_string()).status();
        }
    });

    let output = child.wait_with_output().unwrap();
    let _ = done.send(());
    watchdog.join().unwrap();
    writer.join().unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    output.stdout
}

/// `count` lines of amino acids, 5 to 50 long, from a fixed seed.
fn peptides(count: usize, seed: u64) -> Vec<Vec<u8>> {
    const AMINO_ACIDS: &[u8] = b"ACDEFGHIKLMNPQRSTVWY";

    let mut state = seed;
    let mut next = move || {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    (0..count)
        .map(|_| {
            let length = 5 + (next() % 46) as usize;
            (0..length).map(|_| AMINO_ACIDS[(next() % 20) as usize]).collect()
        })
        .collect()
}

fn join_lines(lines: &[Vec<u8>]) -> Vec<u8> {
    lines.iter().flat_map(|line| line.iter().copied().chain([b'\n'])).collect()
}

/// The lines in byte order, as `LC_ALL=C sort` writes them.
fn sorted(lines: &[Vec<u8>]) -> Vec<u8> {
    let mut lines = lines.to_vec();
    lines.sort();
    join_lines(&lines)
}

#[test]
fn test_sorts_lines_in_byte_order() {
    let lines = peptides(10_000, 1);

    assert_eq!(sorter(&join_lines(&lines), &[]), sorted(&lines));
}

#[test]
fn test_sorts_through_several_merge_levels() {
    // A buffer this small gives hundreds of sorted files, so the files are merged more than once.
    let lines = peptides(50_000, 2);

    for threads in ["1", "4"] {
        assert_eq!(sorter(&join_lines(&lines), &["-b", "20000", "-p", threads]), sorted(&lines));
    }
}

#[test]
fn test_keeps_duplicate_lines() {
    let mut lines = peptides(1_000, 3);
    lines.extend(lines.clone());

    assert_eq!(sorter(&join_lines(&lines), &["-b", "4000"]), sorted(&lines));
}

#[test]
fn test_sorts_from_a_field_to_the_end_of_the_line() {
    // Lines with equal keys can come out in any order, so only the keys and the set of lines are compared.
    let keys = peptides(5_000, 4);
    let values = peptides(5_000, 5);
    let lines: Vec<Vec<u8>> = values.iter().zip(&keys).map(|(value, key)| [&value[..], b"\t", key].concat()).collect();

    let output = sorter(&join_lines(&lines), &["-b", "20000", "-f", "2"]);
    let output_lines: Vec<Vec<u8>> = output.split(|&b| b == b'\n').filter(|l| !l.is_empty()).map(<[u8]>::to_vec).collect();

    let output_keys: Vec<&[u8]> = output_lines.iter().map(|line| &line[line.iter().position(|&b| b == b'\t').unwrap()..]).collect();
    let mut expected_keys: Vec<&[u8]> = lines.iter().map(|line| &line[line.iter().position(|&b| b == b'\t').unwrap()..]).collect();
    expected_keys.sort();

    assert_eq!(output_keys, expected_keys);
    assert_eq!(sorted(&output_lines), sorted(&lines));
}

#[test]
fn test_writes_nothing_for_empty_input() {
    assert_eq!(sorter(b"", &[]), b"");
}

#[test]
fn test_keeps_a_last_line_without_a_newline() {
    assert_eq!(sorter(b"b\na\nc", &[]), b"a\nb\nc\n");
    assert_eq!(sorter(b"x\tb\ny\ta", &["-f", "2"]), b"y\ta\nx\tb\n");
}

#[test]
fn test_sorts_lines_longer_than_the_buffer() {
    // With 4 threads, each chunk gets a quarter of the buffer, so these lines are 12 times longer.
    let mut lines = peptides(200, 6);
    lines.push(vec![b'L'; 3000]);
    lines.push(vec![b'A'; 5000]);

    assert_eq!(sorter(&join_lines(&lines), &["-b", "1000"]), sorted(&lines));
}

#[test]
fn test_keeps_empty_lines() {
    assert_eq!(sorter(b"b\n\na\n\n", &[]), b"\n\na\nb\n");
}

#[test]
fn test_writes_its_temporary_files_in_the_temp_dir() {
    // A directory that does not exist yet, so it only appears when the sorter uses --temp-dir
    let location = std::env::temp_dir().join(format!("extrsort-test-{}", std::process::id())).join("nested");
    let lines = peptides(20_000, 8);

    let output = sorter(&join_lines(&lines), &["-b", "20000", "-t", location.to_str().unwrap()]);

    assert_eq!(output, sorted(&lines));
    assert!(location.is_dir());
    assert_eq!(std::fs::read_dir(&location).unwrap().count(), 0, "the sorter removes its own directory");

    std::fs::remove_dir_all(location.parent().unwrap()).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_fails_when_the_output_cannot_be_written() {
    // /dev/full fails every write. The output fits in the stdout buffer, so only the last flush fails.
    let output = Command::new(env!("CARGO_BIN_EXE_sorter"))
        .stdin(Stdio::piped())
        .stdout(std::fs::File::create("/dev/full").unwrap())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child.stdin.take().unwrap().write_all(b"b\na\n")?;
            child.wait_with_output()
        })
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No space left on device"));
}

/// Starts the sorter with its temporary files in `location`, and gives it half of the input, so it
/// is still running. With `nohup`, it starts with SIGHUP ignored.
fn running_sorter(location: &std::path::Path, nohup: bool) -> (std::process::Child, Vec<Vec<u8>>) {
    let lines = peptides(40_000, 9);
    let (program, args): (&str, Vec<&str>) = if nohup {
        ("nohup", vec![env!("CARGO_BIN_EXE_sorter")])
    } else {
        (env!("CARGO_BIN_EXE_sorter"), vec![])
    };

    let mut child = Command::new(program)
        .args(args)
        .args(["-b", "20000", "-t", location.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(&join_lines(&lines[..20_000])).unwrap();

    // Wait until the sorter has written a sorted chunk into its temporary directory, which it makes
    // before it sets its signal handler
    let chunk_files = || {
        std::fs::read_dir(location)
            .map(|dir| dir.flatten().map(|tmp_dir| std::fs::read_dir(tmp_dir.path()).map(|files| files.count()).unwrap_or(0)).sum())
            .unwrap_or(0)
    };
    let started = std::time::Instant::now();
    while chunk_files() == 0 {
        assert!(started.elapsed().as_secs() < 30, "the sorter wrote no sorted chunk");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    (child, lines)
}

fn signal(child: &std::process::Child, name: &str) {
    assert!(Command::new("kill").args([format!("-{}", name), child.id().to_string()]).status().unwrap().success());
}

#[cfg(unix)]
#[test]
fn test_removes_its_temporary_files_on_sigterm() {
    let location = std::env::temp_dir().join(format!("extrsort-sigterm-{}", std::process::id()));
    let (mut child, _) = running_sorter(&location, false);

    signal(&child, "TERM");
    let status = child.wait().unwrap();

    assert!(!status.success());
    assert_eq!(std::fs::read_dir(&location).unwrap().count(), 0);
    std::fs::remove_dir_all(location).unwrap();
}

#[cfg(unix)]
#[test]
fn test_keeps_sorting_on_sighup_under_nohup() {
    let location = std::env::temp_dir().join(format!("extrsort-sighup-{}", std::process::id()));
    let (mut child, lines) = running_sorter(&location, true);

    signal(&child, "HUP");
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(child.try_wait().unwrap().is_none(), "the sorter stopped on SIGHUP");

    child.stdin.take().unwrap().write_all(&join_lines(&lines[20_000..])).unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, sorted(&lines));
    std::fs::remove_dir_all(location).unwrap();
}
