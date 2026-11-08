use std::io;

fn main() -> io::Result<()> {
    let devices = hom_manager::device::Device::connected()?;
    println!("{} arduino(s) connected:", devices.len());
    for device in devices {
        println!("  {} -> {}", device.id(), device.tty().display());
    }
    Ok(())
}
