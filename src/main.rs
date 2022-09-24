#![allow(unused_imports)]
#![allow(unused_must_use)]

mod diffing;

use diffing::Common;

fn main() {
    let mut args = std::env::args();
    let program = args.nth(0).unwrap();
    let file1 = args.nth(0); 
    let file2 = args.nth(0);
    let opt_arg = args.nth(0);

    if file1.is_none() || file2.is_none() {
        print_usage(program);
        return;
    }
    let mut c_size = 3;
    if let Some(csz) = opt_arg {
        let parsed = csz.parse::<usize>();
        if parsed.is_ok() {
            c_size = parsed.unwrap();
        } else {
            println!("{:?}", parsed);
            print_usage(program);
            return;
        }
    }

    let mut cmn = Common::new(c_size);
    
    let res = cmn.file_to_chunk_hash_list(&file1.unwrap());
    
    if let Some(hash_list) = res {
        if let Some(deltas) = cmn.diff(&hash_list, &file2.unwrap()) {
            // println!("Deltas: {:#?}", deltas);
        }
    }
    
}

fn print_usage(program: String) {
    println!("
    USAGE: {0} <file_1_path> <file_2_path> <optional chunk_size>
    
    Examples:
        {0} abc.txt def.txt
        {0} some.txt other.txt 4
        {0} some.bin other.bin 7
    
    ", program);
}


#[cfg(test)]
mod test_main {
    use super::*;


}