fn main() {
    let info = hardware_info::hw::get_hw_info();
    println!("{info:#?}");
}
