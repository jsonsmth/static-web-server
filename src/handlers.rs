use gotham::handler::assets::FileOptions;
use gotham::middleware::logger::SimpleLogger;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use log::Level;

use crate::config::Options;
use crate::helpers;

pub struct RouterHandler {
    opts: Options,
}

impl RouterHandler {
    /// Create a new instance of `RouterHandler` with given options.
    pub fn new(opts: Options) -> RouterHandler {
        RouterHandler { opts }
    }

    /// Handle the server router configuration
    pub fn handle(&self) -> Router {
        // Setup logging
        let (chain, pipelines) =
            single_pipeline(new_pipeline().add(SimpleLogger::new(Level::Info)).build());

        // Options definition
        let opts = &self.opts;

        // Check the root directory
        let root_dir = match helpers::validate_dirpath(&opts.root) {
            Err(err) => {
                println!("{}", helpers::path_error_fmt(err, "root", &opts.root));
                std::process::exit(1);
            }
            Ok(val) => val,
        };
        // Check the assets directory
        let assets_dir = match helpers::validate_dirpath(&opts.assets) {
            Err(err) => {
                println!("{}", helpers::path_error_fmt(err, "assets", &opts.assets));
                std::process::exit(1);
            }
            Ok(val) => val,
        };

        // Default index html file
        let index_file = format!("{}/index.html", &root_dir.display());

        // Routes configuration (GET & HEAD)
        build_router(chain, pipelines, |route| {
            // Root route configuration
            route.associate("/", |assoc| {
                assoc.head().to_file(&index_file);
                assoc.get().to_file(&index_file);
            });
            // Root wilcard configuration
            route.associate("/*", |assoc| {
                assoc.head().to_dir(
                    FileOptions::new(&root_dir)
                        .with_cache_control("no-cache")
                        .with_gzip(true)
                        .build(),
                );
                assoc.get().to_dir(
                    FileOptions::new(&root_dir)
                        .with_cache_control("no-cache")
                        .with_gzip(true)
                        .build(),
                );
            });

            let assets_dirname = match assets_dir.iter().last() {
                None => {
                    println!("error: assets directory name was not determined");
                    std::process::exit(1);
                }
                Some(val) => val.to_str().unwrap().to_string(),
            };

            // Use assets base directory name as route endpoint
            let assets_route = &format!("/{}/*", assets_dirname);
            route.associate(assets_route, |assoc| {
                assoc.head().to_dir(
                    FileOptions::new(&assets_dir)
                        .with_cache_control("no-cache")
                        .with_gzip(true)
                        .build(),
                );
                assoc.get().to_dir(
                    FileOptions::new(&assets_dir)
                        .with_cache_control("no-cache")
                        .with_gzip(true)
                        .build(),
                );

                // Server info
                let listen = format!("{}{}{}", opts.host.to_string(), ":", opts.port.to_string());
                println!("[INFO] static-web-server listening at {}", &listen);
                println!("[INFO] root endpoint    ->  HEAD,GET  /");
                println!("[INFO] assets endpoint  ->  HEAD,GET  {}", &assets_route);
            });
        })
    }
}