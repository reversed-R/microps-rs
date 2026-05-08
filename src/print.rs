#[macro_export]
macro_rules! dbg {
    ($fmt:literal, $($arg:tt)*) => {
        println!("[dbg] {} ({}:{})", format!($fmt, $($arg)*), file!(), line!());
    };
    ($str:literal) => {
        println!("[dbg] {} ({}:{})", $str, file!(), line!());
    };
}

#[macro_export]
macro_rules! info {
    ($fmt:literal, $($arg:tt)*) => {
        println!("[info] {} ({}:{})", format!($fmt, $($arg)*), file!(), line!());
    };
    ($str:literal) => {
        println!("[info] {} ({}:{})", $str, file!(), line!());
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:literal, $($arg:tt)*) => {
        println!("[warn] {} ({}:{})", format!($fmt, $($arg)*), file!(), line!());
    };
    ($str:literal) => {
        println!("[warn] {} ({}:{})", $str, file!(), line!());
    };
}

#[macro_export]
macro_rules! error {
    ($fmt:literal, $($arg:tt)*) => {
        println!("[error] {} ({}:{})", format!($fmt, $($arg)*), file!(), line!());
    };
    ($str:literal) => {
        println!("[error] {} ({}:{})", $str, file!(), line!());
    };
}

pub(crate) fn debugdump(data: &[u8]) {
    let size = data.len();

    println!("+------+-------------------------------------------------+------------------+");
    for offset in (0..size).step_by(16) {
        print!("| {offset:4x} | ");
        for index in 0..16 {
            if offset + index < size {
                print!("{:02x} ", data[offset + index]);
            } else {
                print!("   ");
            }
        }
        print!("| ");
        for index in 0..16 {
            if offset + index < size {
                if data[offset + index].is_ascii_graphic() {
                    print!("{}", char::from(*data.get(offset + index).unwrap()));
                } else {
                    print!(".");
                }
            } else {
                print!(" ");
            }
        }
        println!(" |");
    }
    println!("+------+-------------------------------------------------+------------------+");
}
