use std::{io::{Write, BufWriter, Read, BufReader}, fs::{File, remove_file}, path::PathBuf};

pub trait TmpFileOpened {
    type Closed: TmpFileClosed;

    fn close(self) -> Self::Closed;
}

pub trait TmpFileClosed {
    type Reopened: TmpFileOpened;

    fn reopen(self) -> Self::Reopened;
}

pub trait TmpFileWrite: TmpFileOpened {
    type InnerWrite: Write;
}

pub trait TmpFileRead: TmpFileOpened + Send {
    type InnerRead: Read;

    fn close_and_remove(self);
}

pub struct ClosedTmpFile {
    path: PathBuf
}

impl TmpFileClosed for ClosedTmpFile {
    type Reopened = TmpFileReader;

    fn reopen(self) -> Self::Reopened {
        let file = BufReader::new(File::open(&self.path).unwrap());
        TmpFileReader { path: self.path, file }
    }
}

pub struct TmpFileWriter {
    path: PathBuf,
    file: BufWriter<File>
}

impl TmpFileWrite for TmpFileWriter {
    type InnerWrite = BufWriter<File>;
}

impl TmpFileOpened for TmpFileWriter {
    type Closed = ClosedTmpFile;

    fn close(self) -> Self::Closed {
        ClosedTmpFile { path: self.path }
    }
}

impl From<PathBuf> for TmpFileWriter {
    fn from(path: PathBuf) -> Self {
        let file = BufWriter::new(File::create(&path).unwrap());
        TmpFileWriter { path, file }
    }
}

impl Write for TmpFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

pub struct TmpFileReader {
    path: PathBuf,
    file: BufReader<File>
}

impl TmpFileRead for TmpFileReader {
    type InnerRead = BufReader<File>;

    fn close_and_remove(self) {
        remove_file(&self.path).unwrap();
    }
}

impl TmpFileOpened for TmpFileReader {
    type Closed = ClosedTmpFile;

    fn close(self) -> Self::Closed {
        ClosedTmpFile { path: self.path }
    }
}

impl From<ClosedTmpFile> for TmpFileReader {
    fn from(closed: ClosedTmpFile) -> Self {
        closed.reopen()
    }
}

impl Read for TmpFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}
