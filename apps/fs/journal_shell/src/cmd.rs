use easy_fs::MAIN_FS;
use libax::fs::{self, File};
use libax::io::{self, prelude::*};
use libax::{string::String, vec::Vec};

macro_rules! print_err {
    ($cmd: literal, $msg: literal) => {
        println!("{}: {}", $cmd, $msg);
    };
    ($cmd: literal, $err: ident) => {
        use io::Error::*;
        println!("{}: {}", $cmd, $err.as_str());
    };
    ($cmd: literal, $arg: expr, $err: ident) => {
        use io::Error::*;
        println!("{}: {}: {}", $cmd, $arg, $err.as_str());
    };
    ($cmd: literal, $arg: expr, $msg: expr) => {
        println!("{}: {}: {}", $cmd, $arg, $msg);
    };
}

type CmdHandler = fn(&str);

const CMD_TABLE: &[(&str, CmdHandler)] = &[
    ("cat", do_cat),
    ("echo", do_echo),
    ("exit", do_exit),
    ("help", do_help),
    ("ls", do_ls),
    ("rm", do_rm),
    ("touch", do_touch),
    ("crash", do_crash),
    ("checkpoint", do_checkpoint),
];

fn do_ls(args: &str) {
    let current_dir = libax::env::current_dir().unwrap();
    let args = if args.is_empty() {
        current_dir.as_str()
    } else {
        args
    };
    let name_count = args.split_whitespace().count();

    fn show_entry_info(path: &str, entry: &str) -> io::Result<()> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        let file_type = metadata.file_type();
        let file_type_char = file_type.as_char();
        let rwx = metadata.permissions().rwx_buf();
        let rwx = unsafe { core::str::from_utf8_unchecked(&rwx) };
        println!("{}{} {:>8} {}", file_type_char, rwx, size, entry);
        Ok(())
    }

    fn list_one(name: &str, print_name: bool) -> io::Result<()> {
        let is_dir = fs::metadata(name)?.is_dir();
        if !is_dir {
            return show_entry_info(name, name);
        }

        if print_name {
            println!("{}:", name);
        }
        let mut entries = fs::read_dir(name)?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect::<Vec<_>>();
        entries.sort();

        for entry in entries {
            let path = String::from(name) + "/" + &entry;
            if let Err(e) = show_entry_info(&path, &entry) {
                print_err!("ls", path, e.as_str());
            }
        }
        Ok(())
    }

    for (i, name) in args.split_whitespace().enumerate() {
        if i > 0 {
            println!();
        }
        if let Err(e) = list_one(name, name_count > 1) {
            print_err!("ls", name, e.as_str());
        }
    }
}

fn do_cat(args: &str) {
    if args.is_empty() {
        print_err!("cat", "no file specified");
        return;
    }

    fn cat_one(fname: &str) -> io::Result<()> {
        let mut buf = [0; 1024];
        let mut file = File::open(fname)?;
        loop {
            let n = file.read(&mut buf)?;
            if n > 0 {
                io::stdout().write(&buf[..n])?;
            } else {
                return Ok(());
            }
        }
    }

    for fname in args.split_whitespace() {
        if let Err(e) = cat_one(fname) {
            print_err!("cat", fname, e.as_str());
        }
    }
}

fn do_echo(args: &str) {
    fn echo_file(fname: &str, text_list: &[&str]) -> io::Result<()> {
        let mut file = File::create(fname)?;
        for text in text_list {
            file.write_all(text.as_bytes())?;
        }
        Ok(())
    }

    if let Some(pos) = args.rfind('>') {
        let text_before = args[..pos].trim();
        let (fname, text_after) = split_whitespace(&args[pos + 1..]);
        if fname.is_empty() {
            print_err!("echo", "no file specified");
            return;
        };

        let text_list = [
            text_before,
            if !text_after.is_empty() { " " } else { "" },
            text_after,
            "\n",
        ];
        if let Err(e) = echo_file(fname, &text_list) {
            print_err!("echo", fname, e.as_str());
        }
    } else {
        println!("{}", args)
    }
}

fn do_touch(args: &str) {
    if args.is_empty() {
        print_err!("touch", "no file specified");
        return;
    }

    fn touch_one(fname: &str) -> io::Result<()> {
        match File::create_new(fname) {
            Err(err) => {
                if err == io::Error::AlreadyExists {
                    Ok(())
                } else {
                    Err(err)
                }
            }
            _ => Ok(()),
        }
    }

    for fname in args.split_whitespace() {
        if let Err(e) = touch_one(fname) {
            print_err!("touch", fname, e.as_str());
        }
    }
}

fn do_rm(args: &str) {
    if args.is_empty() {
        print_err!("rm", "missing operand");
        return;
    }
    let mut rm_dir = false;
    for arg in args.split_whitespace() {
        if arg == "-d" {
            rm_dir = true;
        }
    }

    fn rm_one(path: &str, rm_dir: bool) -> io::Result<()> {
        if rm_dir && fs::metadata(path)?.is_dir() {
            fs::remove_dir(path, false)
        } else {
            fs::remove_file(path)
        }
    }

    for path in args.split_whitespace() {
        if path == "-d" {
            continue;
        }
        if let Err(e) = rm_one(path, rm_dir) {
            print_err!("rm", format_args!("cannot remove '{path}'"), e.as_str());
        }
    }
}

fn do_help(_args: &str) {
    println!("Available commands:");
    for (name, _) in CMD_TABLE {
        println!("  {}", name);
    }
}

fn do_exit(_args: &str) {
    libax::thread::exit(0);
}

fn do_crash(_args: &str) {
    easy_fs::crash();
}

pub fn run_cmd(line: &[u8]) {
    let line_str = unsafe { core::str::from_utf8_unchecked(line) };
    let (cmd, args) = split_whitespace(line_str);
    if !cmd.is_empty() {
        for (name, func) in CMD_TABLE {
            if cmd == *name {
                func(args);
                return;
            }
        }
        println!("{}: command not found", cmd);
    }
}

fn do_checkpoint(_: &str) {
    #[cfg(feature = "journal")]
    unsafe {
        MAIN_FS
            .as_ref()
            .unwrap()
            .as_ref()
            .inner()
            .lock()
            .journal_checkpoint();
    }
}

fn split_whitespace(str: &str) -> (&str, &str) {
    let str = str.trim();
    str.find(char::is_whitespace)
        .map_or((str, ""), |n| (&str[..n], str[n + 1..].trim()))
}
