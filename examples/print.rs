fn main() -> hardware_requiem::Result<()> {
    let info = hardware_requiem::get_hardware_info()?;
    println!("{info:#?}");
    Ok(())
}
