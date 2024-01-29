// AUTOGENERATED DO NOT EDIT

use crate::println;
pub fn print_boot_logo() {
	println!();
	println!();
	println!(r"#######                      #######                               ");
	println!(r"#     # #####  ###### #    #    #    #    # ######  ####  #  ####  ");
	println!(r"#     # #    # #      ##   #    #    #    # #      #      # #      ");
	println!(r"#     # #    # #####  # #  #    #    ###### #####   ####  #  ####  ");
	println!(r"#     # #####  #      #  # #    #    #    # #           # #      # ");
	println!(r"#     # #      #      #   ##    #    #    # #      #    # # #    # ");
	println!(r"####### #      ###### #    #    #    #    # ######  ####  #  ####  ");
	println!(r"                                                                   ");
	println!(r"");
}
pub const SHA: &str = "b5d61269";
pub const PUBLIC_KEY: &[u8] = include_bytes!("../tftp/public_key.pem");
