use std::path::Path;
use bytesize::ByteSize;
use systemstat::{System, Platform};
use termion::{color, style};

const LINE_WIDTH: u64 = 60;
const BAR_WIDTH: u64 = LINE_WIDTH - 2;


fn main() {
    let sys = System::new();

    match sys.mount_at(Path::new("/")) {
        Ok(mount) => {
            let total = mount.total.as_u64();
            let avail = mount.avail.as_u64();
            let used = total - avail;
            let bar_full = BAR_WIDTH * used / total;
            let bar_empty = BAR_WIDTH - bar_full;

            println!("{} -> {} ({}) {}/{}",
                     mount.fs_mounted_from,
                     mount.fs_mounted_on,
                     mount.fs_type,
                     ByteSize::b(used),
                     ByteSize::b(total));
            println!(
                "[{}{}{}{}{}]",
                color::Fg(color::Green),
                "=".repeat(bar_full as usize),
                color::Fg(color::LightBlack),
                "=".repeat(bar_empty as usize),
                style::Reset,
            );
        }
        Err(x) => println!("\nMounts: error: {}", x)
    }
}
