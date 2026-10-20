use std::{path::{PathBuf, Path}, fs::{create_dir_all, remove_dir_all}, process::exit, sync::atomic::{AtomicBool, Ordering}, thread, time::Duration};

use signal_hook::{consts::{SIGINT, SIGTERM}, iterator::Signals};
use tempfile;

use super::tmp_file::TmpFileWriter;

const DEFAULT_TMP_DIR: &str = "/tmp";

/// Set by the signal handler before it removes the temporary directory
static TERMINATING: AtomicBool = AtomicBool::new(false);

pub struct TmpDirBuilder<'a> {
    /// The location of the temporary directory
    location: Option<&'a PathBuf>
}

impl<'a> TmpDirBuilder<'a> {
    pub fn new() -> Self {
        TmpDirBuilder { location: None }
    }

    pub fn with_location(&mut self, location: &'a PathBuf) -> &mut Self {
        self.location = Some(location);
        self
    }

    pub fn build(&mut self) -> TmpDir {
        let default_location = PathBuf::from(DEFAULT_TMP_DIR);
        let location = self.location.unwrap_or_else(|| &default_location);

        // The location may not exist yet, for example the default of --temp-dir
        create_dir_all(location).expect("Failed to create the location of the temporary directory");

        // Create a new temporary directory
        let tmp_dir = tempfile::Builder::new()
            .prefix("extsort")
            .tempdir_in(location)
            .expect("Failed to create temporary directory"); // TODO: map_err

        let tmp_path = tmp_dir.path().to_owned();

        // Remove the temporary files when the program is interrupted or terminated. SIGHUP is not
        // handled, so a sorter started with nohup keeps ignoring it.
        let mut signals = Signals::new([SIGINT, SIGTERM]).expect("Error setting the signal handler");
        std::thread::spawn(move || {
            if signals.forever().next().is_some() {
                TERMINATING.store(true, Ordering::SeqCst);
                delete_tmp_dir_and_files(&tmp_path);
                exit(1);
            }
        });

        TmpDir { tmp_dir, file_count: 0 }
    }
}

pub struct TmpDir {
    /// The temporary directory
    tmp_dir: tempfile::TempDir,

    /// The number of files in the temporary directory
    file_count: usize
}

impl TmpDir {
    pub fn create_new_file(&mut self) -> TmpFileWriter {
        // The signal handler is removing the directory and will exit, so no new file is made
        while TERMINATING.load(Ordering::SeqCst) {
            thread::park();
        }

        let filename = format!("{:0>8}", self.file_count);
        let path = self.tmp_dir.path().join(filename);

        self.file_count += 1;

        path.into()
    }
}

fn delete_tmp_dir_and_files(path: &Path) {
    // All files are made through create_new_file, which stops once TERMINATING is set. A file that
    // was being made at that moment can still appear while the directory is removed, so
    // remove_dir_all is tried again. The caller exits whatever the result.
    for _ in 0..10 {
        if remove_dir_all(path).is_ok() || !path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
}
