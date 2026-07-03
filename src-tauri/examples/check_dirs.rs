use directories::ProjectDirs;

fn main() {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "com.drip.note") {
        println!("config_dir: {:?}", proj_dirs.config_dir());
        println!("data_dir: {:?}", proj_dirs.data_dir());
        println!("data_local_dir: {:?}", proj_dirs.data_local_dir());
        println!("preference_dir: {:?}", proj_dirs.preference_dir());
        
        let data_dir = proj_dirs.data_dir();
        println!("data_dir_parent: {:?}", data_dir.parent());
    } else {
        println!("Could not determine project directories");
    }
}
