use crate::ffi::{OsStr, OsString, c_char};
use crate::fs::TryLockError;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::path::{Path, PathBuf};
pub use crate::sys::fs::common::{copy, exists};
use crate::sys::time::SystemTime;
use crate::sys::{unsupported, unsupported_err};
use crate::fmt;

// Import peos-abi functions (clean names without peos_ prefix)
unsafe extern "C" {
    fn open(path: *const c_char, flags: i32, mode: i32) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    #[allow(dead_code)]
    fn close(fd: i32) -> i32;
}

// File type constants from peos-abi
#[allow(dead_code)]
const DT_DIR: u8 = 4;
#[allow(dead_code)]
const DT_REG: u8 = 8;
#[allow(dead_code)]
const DT_LNK: u8 = 10;
#[allow(dead_code)]
const DT_UNKNOWN: u8 = 0;

// File mode constants
#[allow(dead_code)]
const S_IFMT: u32 = 0o170000;
#[allow(dead_code)]
const S_IFDIR: u32 = 0o040000;
#[allow(dead_code)]
const S_IFREG: u32 = 0o100000;
#[allow(dead_code)]
const S_IFLNK: u32 = 0o120000;

// Open flags
const O_RDONLY: i32 = 0;
const O_WRONLY: i32 = 1;
const O_RDWR: i32 = 2;
const O_APPEND: i32 = 8;
const O_CREAT: i32 = 64;
const O_EXCL: i32 = 128;
const O_TRUNC: i32 = 512;
#[allow(dead_code)]
const O_DIRECTORY: i32 = 65536;

#[repr(C)]
#[derive(Clone)]
struct stat {
    st_mode: u32,
    st_size: i64,
    st_atim: timespec,
    st_mtim: timespec,
    st_ctim: timespec,
}

#[repr(C)]
#[derive(Clone)]
struct timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

#[repr(C)]
#[allow(dead_code)]
struct dirent64 {
    d_ino: u64,
    d_type: u8,
    d_reclen: u16,
    d_name: [u8; 256],
}

#[derive(Debug)]
pub struct File {
    fd: i32,
}

#[derive(Clone)]
pub struct FileAttr {
    #[allow(dead_code)]
    stat: stat,
}

pub struct ReadDir;

pub struct DirEntry;

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FilePermissions {
    readonly: bool,
}

#[derive(Copy, Clone, Eq, Debug, PartialEq)]
pub struct FileType {
    is_dir: bool,
}

impl core::hash::Hash for FileType {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.is_dir.hash(state);
    }
}

#[derive(Debug)]
pub struct DirBuilder;

impl FileAttr {
    pub fn modified(&self) -> io::Result<SystemTime> {
        todo!("FileAttr::modified not implemented for peOS")
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        todo!("FileAttr::accessed not implemented for peOS")
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        todo!("FileAttr::created not implemented for peOS")
    }

    pub fn size(&self) -> u64 {
        0
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions { readonly: false }
    }

    pub fn file_type(&self) -> FileType {
        FileType { is_dir: false }
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.readonly
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }
}

impl FileTimes {
    pub fn set_accessed(&mut self, _t: SystemTime) {}
    pub fn set_modified(&mut self, _t: SystemTime) {}
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    
    pub fn is_file(&self) -> bool {
        !self.is_dir
    }
    
    pub fn is_symlink(&self) -> bool {
        false
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReadDir")
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        None
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        todo!("DirEntry::path not implemented for peOS")
    }

    pub fn file_name(&self) -> OsString {
        todo!("DirEntry::file_name not implemented for peOS")
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        todo!("DirEntry::metadata not implemented for peOS")
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        todo!("DirEntry::file_type not implemented for peOS")
    }

    #[allow(dead_code)]
    pub fn file_name_os_str(&self) -> &OsStr {
        todo!("DirEntry::file_name_os_str not implemented for peOS")
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }
    
    fn get_flags(&self) -> i32 {
        let access = match (self.read, self.write, self.append) {
            (true, false, false) => O_RDONLY,
            (false, true, false) => O_WRONLY,
            (true, true, false) => O_RDWR,
            (false, _, true) => O_WRONLY | O_APPEND,
            (true, _, true) => O_RDWR | O_APPEND,
            _ => O_RDONLY,
        };
        
        let mut flags = access;
        if self.create { flags |= O_CREAT; }
        if self.create_new { flags |= O_CREAT | O_EXCL; }
        if self.truncate { flags |= O_TRUNC; }
        
        flags
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let path_cstr = path.to_str().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?
            .as_bytes();
        let mut path_buf = Vec::with_capacity(path_cstr.len() + 1);
        path_buf.extend_from_slice(path_cstr);
        path_buf.push(0); // null terminator
        
        let flags = opts.get_flags();
        let mode = 0o644;
        
        let fd = unsafe { open(path_buf.as_ptr() as *const c_char, flags, mode) };
        if fd < 0 {
            return Err(io::Error::from_raw_os_error(-fd));
        }
        
        Ok(File { fd })
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        todo!("File::file_attr not implemented for peOS")
    }

    pub fn fsync(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn lock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn lock_shared(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn try_lock(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
    }

    pub fn try_lock_shared(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(unsupported_err()))
    }

    pub fn unlock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        unsupported()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let result = unsafe { read(self.fd, buf.as_mut_ptr(), buf.len()) };
        if result < 0 {
            Err(io::Error::from_raw_os_error(-result as i32))
        } else {
            Ok(result as usize)
        }
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        todo!("File::read_vectored not implemented for peOS")
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_buf(&self, _cursor: BorrowedCursor<'_>) -> io::Result<()> {
        todo!("File::read_buf not implemented for peOS")
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let result = unsafe { write(self.fd, buf.as_ptr(), buf.len()) };
        if result < 0 {
            Err(io::Error::from_raw_os_error(-result as i32))
        } else {
            Ok(result as usize)
        }
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        todo!("File::write_vectored not implemented for peOS")
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&self, _pos: SeekFrom) -> io::Result<u64> {
        todo!("File::seek not implemented for peOS")
    }

    pub fn size(&self) -> Option<io::Result<u64>> {
        None
    }

    pub fn tell(&self) -> io::Result<u64> {
        todo!("File::tell not implemented for peOS")
    }

    pub fn duplicate(&self) -> io::Result<File> {
        unsupported()
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        Ok(())
    }

    pub fn set_times(&self, _times: FileTimes) -> io::Result<()> {
        Ok(())
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder
    }

    pub fn mkdir(&self, _path: &Path) -> io::Result<()> {
        todo!("DirBuilder::mkdir not implemented for peOS")
    }
}

pub fn readdir(_path: &Path) -> io::Result<ReadDir> {
    todo!("readdir not implemented for peOS")
}

pub fn unlink(_path: &Path) -> io::Result<()> {
    todo!("unlink not implemented for peOS")
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    unsupported()
}

pub fn set_perm(_p: &Path, _perm: FilePermissions) -> io::Result<()> {
    Ok(())
}

pub fn rmdir(_path: &Path) -> io::Result<()> {
    todo!("rmdir not implemented for peOS")
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    unsupported()
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn link(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(_path: &Path) -> io::Result<FileAttr> {
    todo!("stat not implemented for peOS")
}

pub fn lstat(_path: &Path) -> io::Result<FileAttr> {
    todo!("lstat not implemented for peOS")
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}