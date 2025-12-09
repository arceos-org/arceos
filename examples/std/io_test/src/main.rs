#[cfg(target_os = "hermit")]
use arceos_rust as _;

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom, BufReader, BufWriter, BufRead};
use std::path::Path;
use std::time::SystemTime;

fn main() -> io::Result<()> {
    println!("=== Rust 文件读写功能测试 ===\n");

    // 测试1: 基本文件写入和读取
    println!("测试1: 基本文件写入和读取");
    basic_file_operations()?;

    // 测试2: 追加写入
    println!("\n测试2: 追加写入");
    append_to_file()?;

    // 测试3: 使用缓冲区读写
    println!("\n测试3: 使用缓冲区读写");
    buffered_io()?;

    // 测试4: 文件指针操作
    println!("\n测试4: 文件指针操作");
    file_seeking()?;

    // 测试5: 文件元数据
    println!("\n测试5: 文件元数据");
    file_metadata()?;

    // 测试6: 错误处理
    println!("\n测试6: 错误处理");
    error_handling()?;

    // 测试7: 读取目录
    println!("\n测试7: 读取目录");
    read_directory()?;

    // 清理测试文件
    cleanup_test_files()?;

    println!("\n=== 所有测试完成 ===");
    Ok(())
}

fn basic_file_operations() -> io::Result<()> {
    let filename = "test_basic.txt";

    // 创建并写入文件
    println!("  写入文件: {}", filename);
    let content = "Hello, Rust file system!\nThis is line 1.\nThis is line 2.\n";
    fs::write(filename, content)?;

    // 读取整个文件
    println!("  读取整个文件:");
    let read_content = fs::read_to_string(filename)?;
    for line in read_content.lines() {
        println!("  {}", line);
    }

    // 使用File结构体读写
    let mut file = File::create("test_write.txt")?;
    // 使用ASCII字符写入
    file.write_all(b"Writing binary data with write_all\n")?;

    // 写入UTF-8字符串
    let utf8_str = "包含UTF-8中文字符的字符串\n";
    file.write_all(utf8_str.as_bytes())?;

    // 读取二进制数据
    let binary_data = fs::read("test_write.txt")?;
    println!("  读取二进制数据长度: {} bytes", binary_data.len());

    // 读取为字符串显示
    match String::from_utf8(binary_data.clone()) {
        Ok(s) => println!("  文件内容:\n  {}", s),
        Err(e) => println!("  包含非UTF-8数据: {}", e),
    }

    Ok(())
}

fn append_to_file() -> io::Result<()> {
    let filename = "test_append.txt";

    // 第一次写入
    fs::write(filename, "Initial content\n")?;

    // 追加写入
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)?;

    file.write_all(b"First appended line\n")?;
    file.write_all(b"Second appended line\n")?;
    file.write_all("第三行追加内容(UTF-8)\n".as_bytes())?;

    // 读取验证
    let content = fs::read_to_string(filename)?;
    println!("  追加后的内容:");
    for line in content.lines() {
        println!("  {}", line);
    }

    Ok(())
}

fn buffered_io() -> io::Result<()> {
    let filename = "test_buffered.txt";

    // 使用BufWriter提高写入性能
    println!("  使用BufWriter写入大量数据");
    let file = File::create(filename)?;
    let mut writer = BufWriter::with_capacity(1024, file);

    for i in 0..10 {
        writer.write_all(format!("Line {}: Some data here\n", i).as_bytes())?;
    }
    writer.flush()?; // 确保缓冲区被清空

    // 使用BufReader读取
    println!("  使用BufReader逐行读取:");
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        if i < 3 { // 只显示前3行
            println!("  Line {}: {}", i, line?.trim());
        }
    }
    println!("  ... (total 10 lines)");

    Ok(())
}

fn file_seeking() -> io::Result<()> {
    let filename = "test_seek.txt";

    // 创建测试文件
    let mut file = File::create(filename)?;
    file.write_all(b"0123456789ABCDEFGHIJ")?;

    // 重新打开文件进行读取
    let mut file = File::open(filename)?;

    // 移动到第5个字节
    file.seek(SeekFrom::Start(5))?;

    let mut buffer = [0; 5];
    file.read_exact(&mut buffer)?;
    println!("  Read 5 bytes from position 5: {}",
             String::from_utf8_lossy(&buffer));

    // 从当前位置向前移动3字节
    file.seek(SeekFrom::Current(3))?;

    let mut buffer = vec![0; 4];
    file.read_exact(&mut buffer)?;
    println!("  Read 4 bytes from current position: {}",
             String::from_utf8_lossy(&buffer));

    // 从文件末尾向前移动8字节
    file.seek(SeekFrom::End(-8))?;

    let mut buffer = vec![0; 3];
    file.read_exact(&mut buffer)?;
    println!("  Read 3 bytes from 8 bytes before end: {}",
             String::from_utf8_lossy(&buffer));

    Ok(())
}

fn file_metadata() -> io::Result<()> {
    let filename = "test_metadata.txt";
    fs::write(filename, "Test file for metadata")?;

    let metadata = fs::metadata(filename)?;

    println!("  File size: {} bytes", metadata.len());
    println!("  Is file: {}", metadata.is_file());
    println!("  Is directory: {}", metadata.is_dir());

    if let Ok(modified) = metadata.modified() {
        let since_epoch = modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        println!("  Last modified: {} seconds since epoch",
                 since_epoch.as_secs());
    }

    if let Ok(created) = metadata.created() {
        let since_epoch = created
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        println!("  Created: {} seconds since epoch",
                 since_epoch.as_secs());
    }

    // 权限信息
    let permissions = metadata.permissions();
    println!("  Readable: {}", !permissions.readonly() || cfg!(windows));

    // 检查文件是否存在
    println!("  File exists: {}", Path::new(filename).exists());

    Ok(())
}

fn error_handling() -> io::Result<()> {
    // 尝试打开不存在的文件
    println!("  测试打开不存在的文件:");
    match File::open("nonexistent_file.txt") {
        Ok(_) => println!("    Error: File should not exist"),
        Err(e) => println!("    Expected error: {}", e),
    }

    // 使用OpenOptions处理不同情况
    println!("\n  OpenOptions测试:");

    // 尝试以只读方式打开不存在的文件
    let result = OpenOptions::new()
        .read(true)
        .open("another_nonexistent.txt");

    match result {
        Ok(_) => println!("    Error: File should not exist"),
        Err(e) => println!("    Expected error: {}", e),
    }

    // 创建文件但避免截断已存在的文件
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)  // 仅当文件不存在时创建
        .open("unique_file.txt");

    match file {
        Ok(mut f) => {
            f.write_all(b"This is a newly created file")?;
            println!("    Successfully created new file");
        }
        Err(e) => println!("    File exists or creation failed: {}", e),
    }

    // 测试文件读写权限
    println!("\n  测试文件权限:");
    let result = OpenOptions::new()
        .read(true)
        .write(true)
        .open("unique_file.txt");

    match result {
        Ok(_) => println!("    File opened for read and write"),
        Err(e) => println!("    Cannot open for read/write: {}", e),
    }

    Ok(())
}

fn read_directory() -> io::Result<()> {
    let dir_path = ".";

    println!("  读取当前目录中的测试文件:");

    let mut test_file_count = 0;

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with("test_") || file_name_str == "unique_file.txt" {
                let metadata = entry.metadata()?;
                let file_type = entry.file_type()?;

                let type_str = if file_type.is_file() {
                    "file"
                } else if file_type.is_dir() {
                    "directory"
                } else {
                    "other"
                };

                println!("    {} ({}，{} bytes)",
                         file_name_str,
                         type_str,
                         metadata.len());

                test_file_count += 1;
            }
        }
    }

    println!("  找到 {} 个测试文件", test_file_count);

    // 创建和删除目录测试
    fs::create_dir_all("test_dir/subdir")?;
    println!("\n    创建目录结构: test_dir/subdir");

    // 在子目录中创建文件
    fs::write("test_dir/subdir/test.txt", "Test file in subdirectory")?;

    // 读取目录内容
    let entries: Vec<_> = fs::read_dir("test_dir")?.collect();
    println!("    test_dir 包含 {} 个条目", entries.len());

    // 递归读取目录
    println!("\n    递归读取目录结构:");
    fn read_dir_recursive(path: &Path, indent: usize) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            let prefix = "  ".repeat(indent);
            if let Some(name) = entry_path.file_name() {
                println!("{}|- {}", prefix, name.to_string_lossy());

                if entry_path.is_dir() {
                    read_dir_recursive(&entry_path, indent + 1)?;
                }
            }
        }
        Ok(())
    }

    if Path::new("test_dir").exists() {
        read_dir_recursive(Path::new("test_dir"), 1)?;
    }

    // 删除目录
    fs::remove_dir_all("test_dir")?;
    println!("    已删除 test_dir 目录");

    Ok(())
}

fn cleanup_test_files() -> io::Result<()> {
    let test_files = [
        "test_basic.txt",
        "test_write.txt",
        "test_append.txt",
        "test_buffered.txt",
        "test_seek.txt",
        "test_metadata.txt",
        "unique_file.txt",
    ];

    println!("\n清理测试文件:");

    for file in &test_files {
        let path = Path::new(file);
        if path.exists() {
            match fs::remove_file(file) {
                Ok(_) => println!("  已删除: {}", file),
                Err(e) => println!("  删除 {} 失败: {}", file, e),
            }
        }
    }

    Ok(())
}