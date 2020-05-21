mod filesystem;
use filesystem::create_data_dir;

fn main() {
    create_data_dir().unwrap();
}
