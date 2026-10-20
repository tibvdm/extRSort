use std::{path::{PathBuf, Path}, fs::{create_dir_all, read_dir, remove_file, remove_dir}, process::exit};

use tempfile;

use super::tmp_file::TmpFileWriter;

const DEFAULT_TMP_DIR: &str = "/tmp";

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

        // Set a handler in case the program is interrupted or terminated (SIGINT, SIGTERM, SIGHUP)
        ctrlc::set_handler(move || {
            delete_tmp_dir_and_files(&tmp_path);
            exit(1);
        }).expect("Error setting Ctrl-C handler"); // TODO: map_err

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
        let filename = format!("{:0>8}", self.file_count);
        let path = self.tmp_dir.path().join(filename);

        self.file_count += 1;

        path.into()
    }
}

fn delete_tmp_dir_and_files(path: &Path) {
    if let Ok(files) = read_dir(path) {
        for file in files.flatten() {
            let _ = remove_file(file.path());
        }
    }
    remove_dir(path).unwrap(); // TODO: map_err
}
