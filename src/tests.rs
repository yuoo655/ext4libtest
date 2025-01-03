use super::*;

#[test]
fn test_open() {
    let disk = Arc::new(Disk {});
    let ext4 = Ext4::open(disk);

    let path = ".";
    let r = ext4.ext4_file_open(path, "r+");
    assert!(r.unwrap() == 2);
    assert!(r.is_ok(), "open directory error {:?}", r.err());

    let path = "./";
    let r = ext4.ext4_file_open(path, "r+");
    assert!(r.unwrap() == 2);
    assert!(r.is_ok(), "open directory error {:?}", r.err());

    let path =
        "test_files/dirtest0/./dirtest1/../dirtest1/../../dirtest0/dirtest1/dirtest2/dirtest3";
    let r = ext4.ext4_file_open(path, "r+");
    assert!(r.is_ok(), "open directory error {:?}", r.err());

    let path = "test_files/dirtest0/./dirtest1/../dirtest1/../../dirtest0/dirtest1/dirtest2/dirtest3/nonexistpath";
    let r = ext4.ext4_file_open(path, "r+");
    assert!(r.is_err());
}

// #[test]
// fn test_read_file() {

//     let disk = Arc::new(Disk {});
//     let ext4 = Ext4::open(disk);

//     // Test reading the file in ext4_rs
//     let file_path_str = "test_files/1.txt";
//     let mut ext4_file = Ext4File::new();
//     let r = ext4.ext4_open(&mut ext4_file, file_path_str, "r+", false);
//     assert!(r.is_ok(), "open file error {:?}", r.err());

//     let mut read_buf = vec![0u8; 0x100000];
//     let mut read_cnt = 0;
//     let r = ext4.ext4_file_read(&mut ext4_file, &mut read_buf, 0x100000, &mut read_cnt);
//     assert!(r.is_ok(), "open file error {:?}", r.err());
//     let data = [0x31u8; 0x100000];
//     assert!(read_buf == data);

// }

// #[test]
// fn test_write_file() {

//     let disk = Arc::new(Disk {});
//     let ext4 = Ext4::open(disk);

//     // dir
//     log::info!("----mkdir----");
//     for i in 0..10 {
//         let path = format!("dirtest{}", i);
//         let path = path.as_str();
//         let r = ext4.ext4_dir_mk(&path);
//         assert!(r.is_ok(), "dir make error {:?}", r.err());
//     }

//     // write test
//     // file
//     log::info!("----write file in dir----");
//     for i in 0..10 {
//         const WRITE_SIZE: usize = 0x400000;
//         let path = format!("dirtest{}/write_{}.txt", i, i);
//         let path = path.as_str();
//         let mut ext4_file = Ext4File::new();
//         let r = ext4.ext4_open(&mut ext4_file, path, "w+", true);
//         assert!(r.is_ok(), "open file error {:?}", r.err());

//         let write_data = vec![0x41 + i as u8; WRITE_SIZE];
//         ext4.ext4_file_write(&mut ext4_file, &write_data, WRITE_SIZE);

//         // test
//         let r = ext4.ext4_open(&mut ext4_file, path, "r+", false);
//         assert!(r.is_ok(), "open file error {:?}", r.err());

//         let mut read_buf = vec![0u8; WRITE_SIZE];
//         let mut read_cnt = 0;
//         let r = ext4.ext4_file_read(&mut ext4_file, &mut read_buf, WRITE_SIZE, &mut read_cnt);
//         assert!(r.is_ok(), "open file error {:?}", r.err());
//         assert_eq!(write_data, read_buf);
//     }
// }
