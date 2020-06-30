//! Virtual pipes.

use crate::handle::{Handle, HandleRights};
use crate::wasi::{types, Errno, Result};
use log::trace;
use std::any::Any;
use std::cell::{Cell, Ref, RefCell};
use std::io::{self, Read, Write};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ReadPipe<R: Read + Any> {
    rights: Rc<Cell<HandleRights>>,
    reader: Rc<RefCell<R>>,
}

impl<R: Read + Any> ReadPipe<R> {
    pub fn new(r: R) -> Self {
        Self::from_shared(Rc::new(RefCell::new(r)))
    }

    pub fn from_shared(reader: Rc<RefCell<R>>) -> Self {
        use types::Rights;
        Self {
            rights: Rc::new(Cell::new(HandleRights::new(
                Rights::FD_DATASYNC
                    | Rights::FD_FDSTAT_SET_FLAGS
                    | Rights::FD_READ
                    | Rights::FD_SYNC
                    | Rights::FD_FILESTAT_GET
                    | Rights::POLL_FD_READWRITE,
                Rights::empty(),
            ))),
            reader,
        }
    }

    pub fn try_into_inner(mut self) -> std::result::Result<R, Self> {
        match Rc::try_unwrap(self.reader) {
            Ok(rc) => Ok(RefCell::into_inner(rc)),
            Err(reader) => {
                self.reader = reader;
                Err(self)
            }
        }
    }
}

impl From<Vec<u8>> for ReadPipe<io::Cursor<Vec<u8>>> {
    fn from(r: Vec<u8>) -> Self {
        Self::new(io::Cursor::new(r))
    }
}

impl From<&[u8]> for ReadPipe<io::Cursor<Vec<u8>>> {
    fn from(r: &[u8]) -> Self {
        Self::from(r.to_vec())
    }
}

impl From<String> for ReadPipe<io::Cursor<String>> {
    fn from(r: String) -> Self {
        Self::new(io::Cursor::new(r))
    }
}

impl From<&str> for ReadPipe<io::Cursor<String>> {
    fn from(r: &str) -> Self {
        Self::from(r.to_string())
    }
}

impl<R: Read + Any> Handle for ReadPipe<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn try_clone(&self) -> io::Result<Box<dyn Handle>> {
        Ok(Box::new(Self {
            rights: self.rights.clone(),
            reader: self.reader.clone(),
        }))
    }

    fn get_file_type(&self) -> types::Filetype {
        types::Filetype::Unknown
    }

    fn get_rights(&self) -> HandleRights {
        self.rights.get()
    }

    fn set_rights(&self, rights: HandleRights) {
        self.rights.set(rights)
    }

    fn advise(
        &self,
        _advice: types::Advice,
        _offset: types::Filesize,
        _len: types::Filesize,
    ) -> Result<()> {
        Err(Errno::Spipe)
    }

    fn allocate(&self, _offset: types::Filesize, _len: types::Filesize) -> Result<()> {
        Err(Errno::Spipe)
    }

    fn fdstat_set_flags(&self, _fdflags: types::Fdflags) -> Result<()> {
        // do nothing for now
        Ok(())
    }

    fn filestat_get(&self) -> Result<types::Filestat> {
        let stat = types::Filestat {
            dev: 0,
            ino: 0,
            nlink: 0,
            size: 0,
            atim: 0,
            ctim: 0,
            mtim: 0,
            filetype: self.get_file_type(),
        };
        Ok(stat)
    }

    fn filestat_set_size(&self, _st_size: types::Filesize) -> Result<()> {
        Err(Errno::Spipe)
    }

    fn preadv(&self, buf: &mut [io::IoSliceMut], offset: types::Filesize) -> Result<usize> {
        if offset != 0 {
            return Err(Errno::Spipe);
        }
        Ok(self.reader.borrow_mut().read_vectored(buf)?)
    }

    fn seek(&self, _offset: io::SeekFrom) -> Result<types::Filesize> {
        Err(Errno::Spipe)
    }

    fn read_vectored(&self, iovs: &mut [io::IoSliceMut]) -> Result<usize> {
        trace!("read_vectored(iovs={:?})", iovs);
        Ok(self.reader.borrow_mut().read_vectored(iovs)?)
    }

    fn create_directory(&self, _path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn openat(
        &self,
        _path: &str,
        _read: bool,
        _write: bool,
        _oflags: types::Oflags,
        _fd_flags: types::Fdflags,
    ) -> Result<Box<dyn Handle>> {
        Err(Errno::Notdir)
    }

    fn link(
        &self,
        _old_path: &str,
        _new_handle: Box<dyn Handle>,
        _new_path: &str,
        _follow: bool,
    ) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn readlink(&self, _path: &str, _buf: &mut [u8]) -> Result<usize> {
        Err(Errno::Notdir)
    }

    fn readlinkat(&self, _path: &str) -> Result<String> {
        Err(Errno::Notdir)
    }

    fn rename(&self, _old_path: &str, _new_handle: Box<dyn Handle>, _new_path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn remove_directory(&self, _path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn symlink(&self, _old_path: &str, _new_path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn unlink_file(&self, _path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }
}

#[derive(Clone, Debug)]
pub struct WritePipe<W: Write + Any> {
    rights: Rc<Cell<HandleRights>>,
    writer: Rc<RefCell<W>>,
}

impl<W: Write + Any> WritePipe<W> {
    pub fn new(w: W) -> Self {
        Self::from_shared(Rc::new(RefCell::new(w)))
    }

    pub fn from_shared(writer: Rc<RefCell<W>>) -> Self {
        use types::Rights;
        Self {
            rights: Rc::new(Cell::new(HandleRights::new(
                Rights::FD_DATASYNC
                    | Rights::FD_FDSTAT_SET_FLAGS
                    | Rights::FD_SYNC
                    | Rights::FD_WRITE
                    | Rights::FD_FILESTAT_GET
                    | Rights::POLL_FD_READWRITE,
                Rights::empty(),
            ))),
            writer,
        }
    }

    pub fn try_into_inner(mut self) -> std::result::Result<W, Self> {
        match Rc::try_unwrap(self.writer) {
            Ok(rc) => Ok(RefCell::into_inner(rc)),
            Err(writer) => {
                self.writer = writer;
                Err(self)
            }
        }
    }
}

impl WritePipe<io::Cursor<Vec<u8>>> {
    pub fn new_in_memory() -> Self {
        Self::new(io::Cursor::new(vec![]))
    }

    pub fn as_slice(&self) -> Ref<[u8]> {
        Ref::map(self.writer.borrow(), |c| c.get_ref().as_slice())
    }
}

impl<W: Write + Any> Handle for WritePipe<W> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn try_clone(&self) -> io::Result<Box<dyn Handle>> {
        Ok(Box::new(Self {
            rights: self.rights.clone(),
            writer: self.writer.clone(),
        }))
    }

    fn get_file_type(&self) -> types::Filetype {
        types::Filetype::Unknown
    }

    fn get_rights(&self) -> HandleRights {
        self.rights.get()
    }

    fn set_rights(&self, rights: HandleRights) {
        self.rights.set(rights)
    }

    fn advise(
        &self,
        _advice: types::Advice,
        _offset: types::Filesize,
        _len: types::Filesize,
    ) -> Result<()> {
        Err(Errno::Spipe)
    }

    fn allocate(&self, _offset: types::Filesize, _len: types::Filesize) -> Result<()> {
        Err(Errno::Spipe)
    }

    fn fdstat_set_flags(&self, _fdflags: types::Fdflags) -> Result<()> {
        // do nothing for now
        Ok(())
    }

    fn filestat_get(&self) -> Result<types::Filestat> {
        let stat = types::Filestat {
            dev: 0,
            ino: 0,
            nlink: 0,
            size: 0,
            atim: 0,
            ctim: 0,
            mtim: 0,
            filetype: self.get_file_type(),
        };
        Ok(stat)
    }

    fn filestat_set_size(&self, _st_size: types::Filesize) -> Result<()> {
        Err(Errno::Spipe)
    }

    fn pwritev(&self, buf: &[io::IoSlice], offset: types::Filesize) -> Result<usize> {
        if offset != 0 {
            return Err(Errno::Spipe);
        }
        Ok(self.writer.borrow_mut().write_vectored(buf)?)
    }

    fn seek(&self, _offset: io::SeekFrom) -> Result<types::Filesize> {
        Err(Errno::Spipe)
    }

    fn write_vectored(&self, iovs: &[io::IoSlice]) -> Result<usize> {
        trace!("write_vectored(iovs={:?})", iovs);
        Ok(self.writer.borrow_mut().write_vectored(iovs)?)
    }

    fn create_directory(&self, _path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn openat(
        &self,
        _path: &str,
        _read: bool,
        _write: bool,
        _oflags: types::Oflags,
        _fd_flags: types::Fdflags,
    ) -> Result<Box<dyn Handle>> {
        Err(Errno::Notdir)
    }

    fn link(
        &self,
        _old_path: &str,
        _new_handle: Box<dyn Handle>,
        _new_path: &str,
        _follow: bool,
    ) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn readlink(&self, _path: &str, _buf: &mut [u8]) -> Result<usize> {
        Err(Errno::Notdir)
    }

    fn readlinkat(&self, _path: &str) -> Result<String> {
        Err(Errno::Notdir)
    }

    fn rename(&self, _old_path: &str, _new_handle: Box<dyn Handle>, _new_path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn remove_directory(&self, _path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn symlink(&self, _old_path: &str, _new_path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }

    fn unlink_file(&self, _path: &str) -> Result<()> {
        Err(Errno::Notdir)
    }
}
