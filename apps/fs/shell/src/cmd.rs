use libax::fs::{read_dir, File, FileType};
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
    ("cd", do_cd),
    ("echo", do_echo),
    ("exit", do_exit),
    ("help", do_help),
    ("ls", do_ls),
    ("mkdir", do_mkdir),
    ("pwd", do_pwd),
    ("rm", do_rm),
    ("uname", do_uname),
];

fn do_ls(args: &str) {
    let mut args = args.trim();
    if args.is_empty() {
        args = ".";
    }
    let name_count = args.split_whitespace().count();

    fn show_entry_info(path: &str, entry: &str) -> io::Result {
        let metadata = File::open(path)?.metadata()?;
        let size = metadata.len();
        let file_type = metadata.file_type();
        let file_type_char = match file_type {
            FileType::Dir => 'd',
            FileType::File => '-',
            FileType::CharDevice => 'c',
            FileType::SymLink => 'l',
            FileType::BlockDevice => 'b',
            _ => '?',
        };
        let rwx = metadata.permissions().rwx_buf();
        let rwx = unsafe { core::str::from_utf8_unchecked(&rwx) };
        println!("{}{} {:>8} {}", file_type_char, rwx, size, entry);
        Ok(())
    }

    let list_one = |name: &str| -> io::Result {
        let is_dir = File::open(name)?.metadata()?.is_dir();
        if !is_dir {
            return show_entry_info(name, name);
        }

        if name_count > 1 {
            println!("{}:", name);
        }
        let mut entries = read_dir(name)?
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
    };

    for (i, name) in args.split_whitespace().enumerate() {
        if i > 0 {
            println!();
        }
        if let Err(e) = list_one(name) {
            print_err!("ls", name, e.as_str());
        }
    }
}

fn do_cat(args: &str) {
    let args = args.trim();
    if args.is_empty() {
        print_err!("cat", "no file specified");
        return;
    }

    fn cat_one(fname: &str) -> io::Result {
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
    println!("{}", args.trim());
}

fn do_mkdir(_args: &str) {
    print_err!("mkdir", Unsupported);
}

fn do_rm(_args: &str) {
    print_err!("rm", Unsupported);
}

fn do_cd(mut args: &str) {
    if args.is_empty() {
        args = "/";
    }
    if !args.contains(char::is_whitespace) {
        if let Err(e) = libax::env::set_current_dir(args) {
            print_err!("cd", args, e.as_str());
        }
    } else {
        print_err!("cd", "too many arguments");
    }
}

fn do_pwd(_args: &str) {
    let pwd = libax::env::current_dir().unwrap();
    println!("{}", pwd);
}

fn do_uname(_args: &str) {
    let arch = option_env!("ARCH").unwrap_or("");
    let platform = option_env!("PLATFORM").unwrap_or("");
    let smp = match option_env!("SMP") {
        None | Some("1") => "",
        _ => " SMP",
    };
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("0.1.0");
    println!(
        "ArceOS {ver}{smp} {arch} {plat}",
        ver = version,
        smp = smp,
        arch = arch,
        plat = platform,
    );
}

fn do_help(_args: &str) {
    println!("Available command:");
    for (name, _) in CMD_TABLE {
        println!("  {}", name);
    }
}

fn do_exit(_args: &str) {
    libax::task::exit(0);
}

pub fn run_cmd(line: &[u8]) {
    let line_str = unsafe { core::str::from_utf8_unchecked(line) }.trim();
    let (cmd, args) = line_str
        .find(' ')
        .map_or((line_str, ""), |n| (&line_str[..n], &line_str[n + 1..]));

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
