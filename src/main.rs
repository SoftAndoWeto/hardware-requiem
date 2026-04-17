fn main() {
    let os_info = hardware_info::get_os_info();
    println!("OS info: {os_info:#?}");
    let hw_info = hardware_info::hw::get_hw_info();
    println!("hw info: {hw_info:#?}");
}
