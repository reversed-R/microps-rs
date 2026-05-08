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
