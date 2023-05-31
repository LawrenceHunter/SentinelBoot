use console::println;

pub const LOGO: &str = r"
 _____                        ______ __                                   
/\  __`\                     /\__  _/\ \                     __           
\ \ \/\ \  _____     __    __\/_/\ \\ \ \___      __    ____/\_\    ____  
 \ \ \ \ \/\ '__`\ /'__`\/' _ `\\ \ \\ \  _ `\  /'__`\ /',__\/\ \  /',__\ 
  \ \ \_\ \ \ \L\ /\  __//\ \/\ \\ \ \\ \ \ \ \/\  __//\__, `\ \ \/\__, `\
   \ \_____\ \ ,__\ \____\ \_\ \_\\ \_\\ \_\ \_\ \____\/\____/\ \_\/\____/
    \/_____/\ \ \/ \/____/\/_/\/_/ \/_/ \/_/\/_/\/____/\/___/  \/_/\/___/ 
             \ \_\                                                        
              \/_/                                                        ";

fn print_initial_boot_log() {
    println!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("{}", crate::helper::LOGO);
}

fn print_platform_info() {
    // Inspired by OpenSBI
    println!("Platform Name             : {}", bsp::name());
    println!("Platform Features         : {}", bsp::unknown());
    println!("Platform HART Count       : {}", bsp::hart_count());
    println!("Platform IPI Device       : {}", bsp::unknown());
    println!("Platform Timer Device     : {}", bsp::unknown());
    println!("Platform Console Device   : {}", bsp::unknown());
    println!("Platform HSM Device       : {}", bsp::unknown());
    println!("Platform Reboot Device    : {}", bsp::unknown());
    println!("Platform Shutdown Device  : {}", bsp::unknown());
    println!("Firmware Base             : {}", bsp::unknown());
    println!("Firmware Size             : {}", bsp::unknown());
    println!("Runtime SBI Version       : {}", bsp::unknown());
}

fn print_domain_info() {
    for domain in 1..1 {
        println!("Domain{} Name             : {}", domain, bsp::unknown());
        println!("Domain{} Boot HART        : {}", domain, bsp::unknown());
        println!("Domain{} HARTs            : {}", domain, bsp::unknown());
        println!("Domain{} RegionXX         : {}", domain, bsp::unknown());
        println!("Domain{} Next Address     : {}", domain, bsp::unknown());
        println!("Domain{} Next Arg1        : {}", domain, bsp::unknown());
        println!("Domain{} Next Mode        : {}", domain, bsp::unknown());
        println!("Domain{} SysReset         : {}", domain, bsp::unknown());
    }
}

fn print_hart_info() {
    println!("Boot HART ID              : {}", bsp::unknown());
    println!("Boot HART Domain          : {}", bsp::unknown());
    println!("Boot HART Priv Version    : {}", bsp::unknown());
    println!("Boot HART Base ISA        : {}", bsp::unknown());
    println!("Boot HART ISA Extensions  : {}", bsp::unknown());
    println!("Boot HART PMP Count       : {}", bsp::unknown());
    println!("Boot HART PMP Granularity : {}", bsp::unknown());
    println!("Boot HART PMP Address Bits: {}", bsp::unknown());
    println!("Boot HART MHPM Count      : {}", bsp::unknown());
    println!("Boot HART MIDELEG         : {}", bsp::unknown());
}

pub fn print_boot_log() {
    print_initial_boot_log();
    println!();
    print_platform_info();
    println!();
    print_domain_info();
    println!();
    print_hart_info();
    println!();
}
