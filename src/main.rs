use ext4_rs::*;
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyWrite, Request, TimeOrNow,
};
use log::{Level, LevelFilter, Metadata, Record};
use std::{
    ffi::OsStr,
    fs::OpenOptions,
    io::{Read, Seek, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

extern crate alloc;
use alloc::sync::Arc;

macro_rules! with_color {
    ($color_code:expr, $($arg:tt)*) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[m", $color_code as u8, format_args!($($arg)*))
    }};
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        let level = record.level();
        let args_color = match level {
            Level::Error => ColorCode::Red,
            Level::Warn => ColorCode::Yellow,
            Level::Info => ColorCode::Green,
            Level::Debug => ColorCode::Cyan,
            Level::Trace => ColorCode::BrightBlack,
        };

        if self.enabled(record.metadata()) {
            println!(
                "{} - {}",
                record.level(),
                with_color!(args_color, "{}", record.args())
            );
        }
    }

    fn flush(&self) {}
}

#[repr(u8)]
enum ColorCode {
    Red = 31,
    Green = 32,
    Yellow = 33,
    Cyan = 36,
    BrightBlack = 90,
}

pub const EPERM: i32 = 1;
pub const ENOENT: i32 = 2;
pub const ESRCH: i32 = 3;
pub const EINTR: i32 = 4;
pub const EIO: i32 = 5;
pub const ENXIO: i32 = 6;
pub const E2BIG: i32 = 7;
pub const ENOEXEC: i32 = 8;
pub const EBADF: i32 = 9;
pub const ECHILD: i32 = 10;
pub const EAGAIN: i32 = 11;
pub const ENOMEM: i32 = 12;
pub const EACCES: i32 = 13;
pub const EFAULT: i32 = 14;
pub const ENOTBLK: i32 = 15;
pub const EBUSY: i32 = 16;
pub const EEXIST: i32 = 17;
pub const EXDEV: i32 = 18;
pub const ENODEV: i32 = 19;
pub const ENOTDIR: i32 = 20;
pub const EISDIR: i32 = 21;
pub const EINVAL: i32 = 22;
pub const ENFILE: i32 = 23;
pub const EMFILE: i32 = 24;
pub const ENOTTY: i32 = 25;
pub const ETXTBSY: i32 = 26;
pub const EFBIG: i32 = 27;
pub const ENOSPC: i32 = 28;
pub const ESPIPE: i32 = 29;
pub const EROFS: i32 = 30;
pub const EMLINK: i32 = 31;
pub const EPIPE: i32 = 32;
pub const EDOM: i32 = 33;
pub const ERANGE: i32 = 34;
pub const EWOULDBLOCK: i32 = EAGAIN;

pub const S_IFIFO: u32 = 4096;
pub const S_IFCHR: u32 = 8192;
pub const S_IFBLK: u32 = 24576;
pub const S_IFDIR: u32 = 16384;
pub const S_IFREG: u32 = 32768;
pub const S_IFLNK: u32 = 40960;
pub const S_IFSOCK: u32 = 49152;
pub const S_IFMT: u32 = 61440;
pub const S_IRWXU: u32 = 448;
pub const S_IXUSR: u32 = 64;
pub const S_IWUSR: u32 = 128;
pub const S_IRUSR: u32 = 256;
pub const S_IRWXG: u32 = 56;
pub const S_IXGRP: u32 = 8;
pub const S_IWGRP: u32 = 16;
pub const S_IRGRP: u32 = 32;
pub const S_IRWXO: u32 = 7;
pub const S_IXOTH: u32 = 1;
pub const S_IWOTH: u32 = 2;
pub const S_IROTH: u32 = 4;
pub const F_OK: i32 = 0;
pub const R_OK: i32 = 4;
pub const W_OK: i32 = 2;
pub const X_OK: i32 = 1;
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;
pub const SIGHUP: i32 = 1;
pub const SIGINT: i32 = 2;
pub const SIGQUIT: i32 = 3;
pub const SIGILL: i32 = 4;
pub const SIGABRT: i32 = 6;
pub const SIGFPE: i32 = 8;
pub const SIGKILL: i32 = 9;
pub const SIGSEGV: i32 = 11;
pub const SIGPIPE: i32 = 13;
pub const SIGALRM: i32 = 14;
pub const SIGTERM: i32 = 15;

const TTL: Duration = Duration::from_secs(1); // 1 second

#[derive(Debug)]
pub struct Disk {}

impl BlockDevice for Disk {
    fn read_offset(&self, offset: usize) -> Vec<u8> {
        // log::info!("read_offset: {:x?}", offset);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("ex4.img")
            .unwrap();
        let mut buf = vec![0u8; BLOCK_SIZE];
        let _ = file.seek(std::io::SeekFrom::Start(offset as u64));
        let _ = file.read_exact(&mut buf);

        buf
    }

    fn write_offset(&self, offset: usize, data: &[u8]) {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("ex4.img")
            .unwrap();

        let _ = file.seek(std::io::SeekFrom::Start(offset as u64));
        let _ = file.write_all(data);
    }
}

struct Ext4Fuse {
    ext4: Ext4,
}

impl Ext4Fuse {
    pub fn new(ext4: Ext4) -> Self {
        Self { ext4 }
    }
}

impl Filesystem for Ext4Fuse {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // log::info!("lookup name {:?}", name);
        // fuse use 1 as root inode
        let parent = match parent {
            // root
            1 => 2,
            _ => parent,
        };

        let r = self.ext4.fuse_lookup(parent, name.to_str().unwrap());

        if r.is_err() {
            reply.error(ENOENT);
            return;
        }

        let file_attr = r.unwrap();

        let file_kind = match file_attr.kind {
            InodeFileType::S_IFREG => FileType::RegularFile,
            InodeFileType::S_IFDIR => FileType::Directory,
            _ => FileType::RegularFile,
        };

        let file_perm = file_attr.perm.bits();

        let attr = FileAttr {
            ino: file_attr.ino,
            size: file_attr.size,
            blocks: file_attr.blocks,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: file_kind,
            perm: file_perm,
            nlink: file_attr.nlink,
            uid: file_attr.uid,
            gid: file_attr.gid,
            rdev: 0,
            flags: 0,
            blksize: BLOCK_SIZE as u32,
        };

        reply.entry(&TTL, &attr, 0);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        // log::info!("get attr {:x?}", ino);
        let inode = match ino {
            // root
            1 => 2,
            _ => ino,
        };

        let r = self.ext4.fuse_getattr(inode);

        if r.is_err() {
            reply.error(ENOENT);
            return;
        }

        let file_attr = r.unwrap();

        let file_kind = match file_attr.kind {
            InodeFileType::S_IFREG => FileType::RegularFile,
            InodeFileType::S_IFDIR => FileType::Directory,
            _ => FileType::RegularFile,
        };

        let file_perm = file_attr.perm.bits();

        let attr = FileAttr {
            ino: file_attr.ino,
            size: file_attr.size,
            blocks: file_attr.blocks,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: file_kind,
            perm: file_perm,
            nlink: file_attr.nlink,
            uid: file_attr.uid,
            gid: file_attr.gid,
            rdev: 0,
            flags: 0,
            blksize: BLOCK_SIZE as u32,
        };

        reply.attr(&TTL, &attr);
    }

    fn setattr(
        &mut self,
        _req: &Request,
        inode: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<TimeOrNow>,
        mtime: Option<TimeOrNow>,
        ctime: Option<SystemTime>,
        fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        let inode = match inode {
            // root
            1 => 2,
            _ => inode,
        };

        let now = system_time_to_secs(SystemTime::now());

        let mut atime_secs = None;
        if let Some(atime) = atime {
            let secs = match atime {
                TimeOrNow::SpecificTime(t) => system_time_to_secs(t),
                TimeOrNow::Now => now,
            };
            atime_secs = Some(secs);
        }

        let mut mtime_secs = None;
        if let Some(mtime) = mtime {
            let secs = match mtime {
                TimeOrNow::SpecificTime(t) => system_time_to_secs(t),
                TimeOrNow::Now => now,
            };
            mtime_secs = Some(secs);
        }

        let mut ctime_secs = None;
        if let Some(ctime) = ctime {
            let secs = system_time_to_secs(ctime);
            ctime_secs = Some(secs);
        }

        let mut crtime_secs = None;
        if let Some(crtime) = crtime {
            let secs = system_time_to_secs(crtime);
            crtime_secs = Some(secs);
        }

        let mut chgtime_secs = None;
        if let Some(chgtime) = chgtime {
            let secs = system_time_to_secs(chgtime);
            chgtime_secs = Some(secs);
        }

        let mut bkuptime_secs = None;
        if let Some(bkuptime) = bkuptime {
            let secs = system_time_to_secs(bkuptime);
            bkuptime_secs = Some(secs);
        }

        self.ext4.fuse_setattr(
            inode,
            mode,
            uid,
            gid,
            size,
            atime_secs,
            mtime_secs,
            ctime_secs,
            fh,
            crtime_secs,
            chgtime_secs,
            bkuptime_secs,
            flags,
        );

        let r = self.ext4.fuse_getattr(inode);
        if r.is_err() {
            reply.error(EIO);
            return;
        }
        let file_attr = r.unwrap();

        let file_kind = match file_attr.kind {
            InodeFileType::S_IFREG => FileType::RegularFile,
            InodeFileType::S_IFDIR => FileType::Directory,
            _ => FileType::RegularFile,
        };

        let file_perm = file_attr.perm.bits();

        let response_attr = FileAttr {
            ino: file_attr.ino,
            size: file_attr.size,
            blocks: file_attr.blocks,
            atime: timestamp_to_system_time(file_attr.atime),
            mtime: timestamp_to_system_time(file_attr.mtime),
            ctime: timestamp_to_system_time(file_attr.ctime),
            crtime: timestamp_to_system_time(file_attr.crtime),
            kind: file_kind,
            perm: file_perm,
            nlink: file_attr.nlink,
            uid: file_attr.uid,
            gid: file_attr.gid,
            rdev: 0,
            flags: 0,
            blksize: BLOCK_SIZE as u32,
        };

        reply.attr(&Duration::from_secs(1), &response_attr); // 缓存时间可调整
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock: Option<u64>,
        reply: ReplyData,
    ) {
        let inode = match ino {
            // root
            1 => 2,
            _ => ino,
        };
        let r = self.ext4.fuse_read(inode, fh, offset, size, flags, lock);
        match r {
            Ok(data) => reply.data(&data),
            Err(_) => reply.error(ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let inode = match ino {
            // root
            1 => 2,
            _ => ino,
        };

        let r = self.ext4.fuse_readdir(inode, fh, offset);
        match r {
            Ok(entries) => {
                for (i, entry) in entries.iter().enumerate().skip(offset as usize) {
                    let name = entry.get_name();
                    let detype = entry.get_de_type();
                    let kind = match detype {
                        1 => FileType::RegularFile,
                        2 => FileType::Directory,
                        _ => FileType::RegularFile,
                    };
                    let _ = reply.add(entry.inode as u64, (i + 1) as i64, kind, &name);
                }
                reply.ok();
            }
            Err(_) => reply.error(ENOENT),
        }
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        log::info!("write {:?}", ino);
        let inode = match ino {
            // root
            1 => 2,
            _ => ino,
        };

        let r = self
            .ext4
            .fuse_write(inode, fh, offset, data, write_flags, flags, lock_owner);
        match r {
            Ok(size) => reply.written(size as u32),
            Err(_) => reply.error(ENOENT),
        }
    }

    /// Remove a file.
    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let parent = match parent {
            // root
            1 => 2,
            _ => parent,
        };

        let r = self.ext4.fuse_unlink(parent, name.to_str().unwrap());
        match r {
            Ok(_) => reply.ok(),
            Err(_) => reply.error(ENOENT),
        }
    }

    /// Create file node.
    /// Create a regular file, character device, block device, fifo or socket node.
    fn mknod(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: ReplyEntry,
    ) {
        let parent = match parent {
            // root
            1 => 2,
            _ => parent,
        };

        let r = self.ext4.fuse_mknod_with_attr(
            parent,
            name.to_str().unwrap(),
            mode,
            umask,
            rdev,
            _req.uid(),
            _req.gid(),
        );

        match r {
            Ok(inode_ref) => {
                let inode_num = inode_ref.inode_num;
                let attr = FileAttr {
                    ino: inode_num as u64,
                    size: 0,
                    blocks: 0,
                    atime: UNIX_EPOCH,
                    mtime: UNIX_EPOCH,
                    ctime: UNIX_EPOCH,
                    crtime: UNIX_EPOCH,
                    kind: FileType::RegularFile,
                    perm: inode_ref.inode.file_perm().bits(),
                    nlink: 1,
                    uid: _req.uid(),
                    gid: _req.gid(),
                    rdev,
                    flags: 0,
                    blksize: BLOCK_SIZE as u32,
                };

                reply.entry(&TTL, &attr, 0);
            }
            Err(_) => {
                reply.error(ENOENT);
            }
        }
    }

    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        // log::info!("mkdir name {:?} mode {:x?}", name, mode);
        let parent = match parent {
            // root
            1 => 2,
            _ => parent,
        };

        let inode_ref = self
            .ext4
            .fuse_mkdir_with_attr(
                parent,
                name.to_str().unwrap(),
                mode,
                umask,
                _req.uid(),
                _req.gid(),
            )
            .unwrap();

        let inode_num = inode_ref.inode_num;
        let attr = FileAttr {
            ino: inode_num as u64,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o777,
            nlink: 2,
            uid: _req.uid(),
            gid: _req.gid(),
            rdev: 0,
            flags: 0,
            blksize: BLOCK_SIZE as u32,
        };

        reply.entry(&TTL, &attr, 0);
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        // log::info!("remove dir {:?}", name);
        let parent = match parent {
            // root
            1 => 2,
            _ => parent,
        };

        let r = self.ext4.fuse_rmdir(parent, name.to_str().unwrap());
        match r {
            Ok(_) => reply.ok(),
            Err(_) => reply.error(ENOENT),
        }
    }
}

// fn time_now() -> (i64, u32) {
//     time_from_system_time(&SystemTime::now())
// }
//
// fn time_from_system_time(system_time: &SystemTime) -> (i64, u32) {
//     // Convert to signed 64-bit time with epoch at 0
//     match system_time.duration_since(UNIX_EPOCH) {
//         Ok(duration) => (duration.as_secs() as i64, duration.subsec_nanos()),
//         Err(before_epoch_error) => (
//             -(before_epoch_error.duration().as_secs() as i64),
//             before_epoch_error.duration().subsec_nanos(),
//         ),
//     }
// }
//
// fn system_time_from_time(secs: i64, nsecs: u32) -> SystemTime {
//     if secs >= 0 {
//         UNIX_EPOCH + Duration::new(secs as u64, nsecs)
//     } else {
//         UNIX_EPOCH - Duration::new((-secs) as u64, nsecs)
//     }
// }
//
// fn system_time_to_timestamp(time: SystemTime) -> u32 {
//     time.duration_since(UNIX_EPOCH)
//         .unwrap_or(Duration::from_secs(0))
//         .as_secs() as u32
// }

fn timestamp_to_system_time(timestamp: u32) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(timestamp as u64)
}

fn system_time_to_secs(time: SystemTime) -> u32 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs() as u32
}
use std::env;

fn main() {
    log::set_logger(&SimpleLogger).unwrap();
    log::set_max_level(LevelFilter::Info);

    let disk = Arc::new(Disk {});
    let ext4 = Ext4::open(disk);
    let ext4_fuse = Ext4Fuse::new(ext4);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("No mount point specified!");
    }
    let mountpoint = &args[1];

    let mut options = vec![
        MountOption::RW,
        MountOption::FSName("ext4_test".to_string()),
    ];

    options.push(MountOption::AutoUnmount);
    options.push(MountOption::AllowRoot);
    fuser::mount2(ext4_fuse, mountpoint, &options).unwrap();
}

#[cfg(test)]
mod tests;
