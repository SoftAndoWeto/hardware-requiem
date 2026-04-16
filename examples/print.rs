fn main() -> hardware_info::Result<()> {
    let info = hardware_info::get_hardware_info()?;
    println!("{info:#?}");
    Ok(())
}
