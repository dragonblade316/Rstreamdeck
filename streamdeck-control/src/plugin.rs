use abi_stable::library::RootModule; 
use anyhow::Result; 


fn load_plugins() -> Result<Box<dyn Rstreamdeck_lib::Button>> {
    let home_dirs = xdg::BaseDirectories::with_prefix("rstreamdeck").unwrap();
    let dir = home_dirs.create_config_directory("plugins")?; 
    
    let dyn_libs = home_dirs.find_config_files("plugins");

    for i in dyn_libs {
        let plugin = Rstreamdeck_lib::Plugin::load_from_file(i);

        

        


    }
    unimplemented!()
}
