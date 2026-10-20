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
