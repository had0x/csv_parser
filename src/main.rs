use csv_parser::parser::parse_csv;
use std::env;
use std::io::ErrorKind;
use std::path::Path;

///main function, can throw io error.
fn main() -> Result<(), std::io::Error> {
    //get a list of arguments passed to our program
    let arguments: Vec<String> = env::args().collect();

    //println!("{:#?}", arguments);

    //check if we actualy got some args or nothing was provided
    //we always get a first argument (this is the path of the binary)
    if arguments.len() == 1 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Cannot continue, please provide a file name for parsing.",
        ));
    }

    //get first argument (second one from arguments vector) and check it's validity (we ignore any arguments passed after this one)
    let first_arg = &arguments[1];

    //check to see if it has a csv extension format (before checking if the file is on disk)
    if !first_arg.ends_with(".csv") {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            "Cannot continue, file extension must end with '.csv'.",
        ));
    }
    //check if file is on disk an readable, dont open it yet, maybe we can stream it later
    let file_path = Path::new(first_arg);
    if !file_path.exists() {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            format!("Cannot continue, file '{}' does not exist.", first_arg),
        ));
    }

    match file_path.metadata() {
        Ok(meta) => {
            if !meta.is_file() {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Cannot continue, '{}' is not a file.", first_arg),
                ));
            }
        }
        Err(_) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!(
                    "Cannot continue, cannot access file '{}' metadata.",
                    first_arg
                ),
            ));
        }
    }

    //call parse_csv function from module and return result in main
    return parse_csv(first_arg);
}
